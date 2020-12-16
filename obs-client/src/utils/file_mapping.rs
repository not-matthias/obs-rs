use std::ops::{Deref, DerefMut};
use winapi::um::{
    handleapi::CloseHandle,
    memoryapi::{MapViewOfFile, UnmapViewOfFile, FILE_MAP_ALL_ACCESS},
    winbase::OpenFileMappingA,
};

pub struct FileMapping<T> {
    handle: usize,
    file_mapping: *mut T,
}

impl<T> FileMapping<T> {
    pub fn open<S: AsRef<str>>(name: S) -> Option<Self> {
        log::info!("Trying to create a file mapping to {:?}.", name.as_ref());

        let handle = unsafe {
            OpenFileMappingA(
                FILE_MAP_ALL_ACCESS,
                false as _,
                format!("{}\0", name.as_ref()).as_ptr() as _,
            )
        };
        if handle.is_null() {
            log::warn!("Failed to open file mapping ({:?}).", name.as_ref());
            return None;
        }

        let file_mapping =
            unsafe { MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, std::mem::size_of::<T>()) } as *mut T;
        if file_mapping.is_null() {
            log::warn!("Failed to map view of file ({:?}).", name.as_ref());
            return None;
        }

        Some(Self {
            handle: handle as usize,
            file_mapping,
        })
    }
}

impl<T> Deref for FileMapping<T> {
    type Target = *mut T;

    fn deref(&self) -> &Self::Target { &(self.file_mapping) }
}

impl<T> DerefMut for FileMapping<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut (self.file_mapping) }
}

impl<T> Drop for FileMapping<T> {
    fn drop(&mut self) {
        unsafe { UnmapViewOfFile(self.file_mapping as _) };
        unsafe { CloseHandle(self.handle as _) };
    }
}
