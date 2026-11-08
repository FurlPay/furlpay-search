//! Atomic SPSC ring buffer with memory barriers.

use std::sync::atomic::{AtomicU64, Ordering};

const CACHE_LINE: usize = 64;

/// Header of the ring buffer in shared memory.
#[repr(C, align(64))]
pub struct RingBufferHeader {
    pub write_pos: AtomicU64,
    _pad1: [u8; CACHE_LINE - 8],
    pub read_pos: AtomicU64,
    _pad2: [u8; CACHE_LINE - 8],
    pub capacity: u64,
    pub slot_size: u64,
}

impl RingBufferHeader {
    pub fn available(&self) -> u64 {
        let w = self.write_pos.load(Ordering::Acquire);
        let r = self.read_pos.load(Ordering::Acquire);
        w.wrapping_sub(r)
    }

    pub fn is_empty(&self) -> bool {
        self.available() == 0
    }

    pub fn is_full(&self) -> bool {
        self.available() >= self.capacity / self.slot_size
    }
}

/// Producer-side handle.
pub struct RingProducer {
    header: *mut RingBufferHeader,
    data: *mut u8,
    slot_count: u64,
    slot_size: u64,
}

impl RingProducer {
    /// # Safety
    /// Base must be valid, aligned, and only one producer per ring.
    pub unsafe fn new(base: *mut u8, capacity: usize, slot_size: usize) -> Self {
        let header = base as *mut RingBufferHeader;
        let data = base.add(std::mem::size_of::<RingBufferHeader>());
        let slot_count = capacity as u64 / slot_size as u64;

        (*header).capacity = capacity as u64;
        (*header).slot_size = slot_size as u64;

        Self {
            header,
            data,
            slot_count,
            slot_size: slot_size as u64,
        }
    }

    pub fn try_write(&self, message: &[u8]) -> bool {
        let header = unsafe { &*self.header };
        if message.len() > self.slot_size as usize {
            return false;
        }

        let write = header.write_pos.load(Ordering::Relaxed);
        let read = header.read_pos.load(Ordering::Acquire);

        if write.wrapping_sub(read) >= self.slot_count {
            return false;
        }

        let slot_index = (write % self.slot_count) as usize;
        let offset = slot_index * self.slot_size as usize;

        unsafe {
            let dst = self.data.add(offset);
            std::ptr::copy_nonoverlapping(message.as_ptr(), dst, message.len());
            if message.len() < self.slot_size as usize {
                std::ptr::write_bytes(
                    dst.add(message.len()),
                    0,
                    self.slot_size as usize - message.len(),
                );
            }
        }

        header
            .write_pos
            .store(write.wrapping_add(1), Ordering::Release);
        true
    }
}

unsafe impl Send for RingProducer {}

/// Consumer-side handle.
pub struct RingConsumer {
    header: *const RingBufferHeader,
    data: *const u8,
    slot_count: u64,
    slot_size: u64,
}

impl RingConsumer {
    /// # Safety
    /// Base must be valid, aligned, and only one consumer per ring.
    pub unsafe fn new(base: *const u8, capacity: usize, slot_size: usize) -> Self {
        let header = base as *const RingBufferHeader;
        let data = base.add(std::mem::size_of::<RingBufferHeader>());

        Self {
            header,
            data,
            slot_count: capacity as u64 / slot_size as u64,
            slot_size: slot_size as u64,
        }
    }

    pub fn try_read(&self) -> Option<Vec<u8>> {
        let header = unsafe { &*self.header };
        let write = header.write_pos.load(Ordering::Acquire);
        let read = header.read_pos.load(Ordering::Relaxed);

        if read >= write {
            return None;
        }

        let slot_index = (read % self.slot_count) as usize;
        let offset = slot_index * self.slot_size as usize;

        let data = unsafe {
            let src = self.data.add(offset);
            std::slice::from_raw_parts(src, self.slot_size as usize).to_vec()
        };

        header
            .read_pos
            .store(read.wrapping_add(1), Ordering::Release);
        Some(data)
    }

    pub fn has_data(&self) -> bool {
        let header = unsafe { &*self.header };
        !header.is_empty()
    }
}

unsafe impl Send for RingConsumer {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic() {
        let slot_size = 512;
        let slot_count = 16;
        let capacity = slot_size * slot_count;
        let total_size = std::mem::size_of::<RingBufferHeader>() + capacity;

        let mut buffer = vec![0u8; total_size + 64];
        let base = buffer.as_mut_ptr();
        let aligned = ((base as usize + 63) & !63) as *mut u8;

        unsafe {
            let producer = RingProducer::new(aligned, capacity, slot_size);
            let consumer = RingConsumer::new(aligned as *const u8, capacity, slot_size);

            let msg = b"hello furlpay";
            assert!(producer.try_write(msg));

            let received = consumer.try_read().expect("should have data");
            assert_eq!(&received[..msg.len()], msg);
        }
    }
}
