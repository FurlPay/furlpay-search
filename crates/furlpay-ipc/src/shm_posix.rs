//! Linux/macOS shared memory implementation using shm_open/mmap.

#[cfg(unix)]
use std::ffi::CString;

#[cfg(unix)]
pub fn create_shared_memory(
    name: &str,
    size: usize,
) -> std::io::Result<super::SharedMemoryRegion> {
    let c_name =
        CString::new(name).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    unsafe {
        let fd = libc::shm_open(c_name.as_ptr(), libc::O_CREAT | libc::O_RDWR, 0o600);
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }

        if libc::ftruncate(fd, size as libc::off_t) != 0 {
            libc::close(fd);
            return Err(std::io::Error::last_os_error());
        }

        let ptr = libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            0,
        );
        libc::close(fd);

        if ptr == libc::MAP_FAILED {
            return Err(std::io::Error::last_os_error());
        }

        Ok(super::SharedMemoryRegion::new(
            ptr as *mut u8,
            size,
            name.to_string(),
        ))
    }
}
