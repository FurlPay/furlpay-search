//! Windows shared memory implementation using CreateFileMappingW/MapViewOfFile.

#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
#[cfg(windows)]
use windows_sys::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile, FILE_MAP_ALL_ACCESS, PAGE_READWRITE,
};

#[cfg(windows)]
pub fn create_shared_memory(
    name: &str,
    size: usize,
) -> std::io::Result<super::SharedMemoryRegion> {
    let wide_name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let handle = CreateFileMappingW(
            INVALID_HANDLE_VALUE,
            std::ptr::null(),
            PAGE_READWRITE,
            (size >> 32) as u32,
            size as u32,
            wide_name.as_ptr(),
        );

        if handle.is_null() {
            return Err(std::io::Error::last_os_error());
        }

        let view = MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, size);

        if view.Value.is_null() {
            CloseHandle(handle);
            return Err(std::io::Error::last_os_error());
        }

        CloseHandle(handle);

        Ok(super::SharedMemoryRegion::new(
            view.Value as *mut u8,
            size,
            name.to_string(),
        ))
    }
}
