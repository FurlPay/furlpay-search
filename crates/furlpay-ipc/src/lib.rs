//! FurlPay IPC — Zero-copy shared-memory ring buffer for daemon–client communication.

pub mod protocol;
pub mod ring_buffer;
pub mod broker;

#[cfg(unix)]
pub mod shm_posix;

#[cfg(windows)]
pub mod shm_windows;

/// Platform-agnostic shared memory handle.
pub struct SharedMemoryRegion {
    ptr: *mut u8,
    len: usize,
    name: String,
}

impl SharedMemoryRegion {
    /// # Safety
    /// The caller must ensure ptr is valid for `len` bytes.
    pub unsafe fn new(ptr: *mut u8, len: usize, name: String) -> Self {
        Self { ptr, len, name }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

unsafe impl Send for SharedMemoryRegion {}
