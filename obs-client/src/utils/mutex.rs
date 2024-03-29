use winapi::um::{errhandlingapi::GetLastError, handleapi::CloseHandle, synchapi::CreateMutexA};

pub struct Mutex {
    handle: usize,
}

impl Mutex {
    pub fn create<S: AsRef<str>>(name: S) -> Option<Self> {
        let handle = unsafe {
            CreateMutexA(
                std::ptr::null_mut(),
                false as _,
                format!("{}\0", name.as_ref()).as_ptr() as _,
            )
        };
        if handle.is_null() {
            log::warn!("Failed to create mutex ({:?}, {:?})", name.as_ref(), unsafe {
                GetLastError()
            });
            None
        } else {
            log::trace!("Created the mutex {:?} = 0x{:x}", name.as_ref(), handle as usize);
            Some(Self { handle: handle as _ })
        }
    }
}

impl Drop for Mutex {
    fn drop(&mut self) {
        log::trace!("Dropping the mutex");
        unsafe { CloseHandle(self.handle as _) };
    }
}
