use crate::Event;
use std::{
    mem::MaybeUninit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
};
use winapi::{
    shared::winerror::ERROR_IO_PENDING,
    um::{
        errhandlingapi::GetLastError,
        fileapi::ReadFile,
        handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
        ioapiset::GetOverlappedResult,
        minwinbase::{OVERLAPPED, SECURITY_ATTRIBUTES},
        namedpipeapi::ConnectNamedPipe,
        securitybaseapi::{InitializeSecurityDescriptor, SetSecurityDescriptorDacl},
        synchapi::WaitForSingleObject,
        winbase::{
            CreateNamedPipeA, FILE_FLAG_OVERLAPPED, INFINITE, PIPE_ACCESS_DUPLEX, PIPE_READMODE_MESSAGE,
            PIPE_TYPE_MESSAGE, PIPE_WAIT, WAIT_OBJECT_0,
        },
        winnt::{SECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_REVISION},
    },
};

pub const IPC_PIPE_BUFFER_SIZE: u32 = 1024;

pub struct NamedPipe {
    handle: usize,

    _thread: JoinHandle<()>,
    thread_running: Arc<AtomicBool>,
}

impl NamedPipe {
    fn create_events() -> Option<Event> { Event::create(None) }

    fn create_full_access_security_descriptor() -> Option<SECURITY_DESCRIPTOR> {
        let mut sd = MaybeUninit::<SECURITY_DESCRIPTOR>::uninit();

        if unsafe { InitializeSecurityDescriptor(sd.as_mut_ptr() as _, SECURITY_DESCRIPTOR_REVISION) == 0 } {
            return None;
        }

        if unsafe { SetSecurityDescriptorDacl(sd.as_mut_ptr() as _, true as _, std::ptr::null_mut(), false as _) == 0 }
        {
            return None;
        }

        Some(unsafe { sd.assume_init() })
    }

    fn create_pipe<S: AsRef<str>>(name: S) -> Option<usize> {
        let mut sd = Self::create_full_access_security_descriptor()?;
        let mut sa: SECURITY_ATTRIBUTES = unsafe { core::mem::zeroed() };

        sa.nLength = core::mem::size_of::<SECURITY_ATTRIBUTES>() as _;
        sa.lpSecurityDescriptor = &mut sd as *mut _ as _;
        sa.bInheritHandle = false as _;

        let pipe_name = format!("\\\\.\\pipe\\{}\0", name.as_ref());
        let handle = unsafe {
            CreateNamedPipeA(
                pipe_name.as_ptr() as _,
                PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                1,
                IPC_PIPE_BUFFER_SIZE,
                IPC_PIPE_BUFFER_SIZE,
                0,
                &mut sa as *mut _,
            )
        };

        std::mem::drop(sd);

        if handle != INVALID_HANDLE_VALUE {
            log::trace!("Created the named pipe {:?} = 0x{:x}", name.as_ref(), handle as usize);
            Some(handle as usize)
        } else {
            None
        }
    }

    fn wait_for_connection(pipe_handle: usize, event_handle: usize) -> Option<OVERLAPPED> {
        let mut overlap: OVERLAPPED = unsafe { std::mem::zeroed() };
        overlap.hEvent = event_handle as _;

        let result = unsafe { ConnectNamedPipe(pipe_handle as _, &mut overlap as _) };
        if result != 0 || (result == 0 && Self::io_pending()) {
            Some(overlap)
        } else {
            None
        }
    }

    fn io_pending() -> bool { unsafe { GetLastError() == ERROR_IO_PENDING } }

    pub fn create<S: AsRef<str>>(name: S) -> Option<Self> {
        let ready_event = Self::create_events()?;
        let pipe_handle = Self::create_pipe(name)? as usize;

        let thread_running = Arc::new(AtomicBool::new(true));
        let thread_running_copy = thread_running.clone();

        // Create the read thread
        //
        log::info!("Creating the thread");
        let thread_handle = std::thread::spawn(move || {
            // Initialize the overlap struct
            //
            let mut overlap = if let Some(overlap) = Self::wait_for_connection(pipe_handle, ready_event.handle()) {
                overlap
            } else {
                log::warn!("Self::wait_for_connection failed");
                return;
            };
            let ready_event = ready_event;

            // Wait for connection
            //
            let wait = unsafe { WaitForSingleObject(ready_event.handle() as _, INFINITE) };
            if wait != WAIT_OBJECT_0 {
                log::warn!("wait != WAIT_OBJECT_0");
                return;
            }

            //
            //
            let mut buffer: Vec<u8> = Vec::with_capacity(IPC_PIPE_BUFFER_SIZE as _);
            let mut temp: [u8; IPC_PIPE_BUFFER_SIZE as _] = [0_u8; IPC_PIPE_BUFFER_SIZE as _];
            while thread_running.load(Ordering::Relaxed) {
                // Read to the buffer
                //
                if unsafe {
                    ReadFile(
                        pipe_handle as _,
                        temp.as_mut_ptr() as _,
                        IPC_PIPE_BUFFER_SIZE,
                        std::ptr::null_mut(),
                        &mut overlap,
                    )
                } != 0
                    && !Self::io_pending()
                {
                    log::warn!("ReadFile failed ({:?})", unsafe { GetLastError() });
                    break;
                }

                if unsafe { WaitForSingleObject(ready_event.handle() as _, INFINITE) } != WAIT_OBJECT_0 {
                    log::warn!("WaitForSingleObject failed");
                    break;
                }

                let mut bytes = 0;
                let success =
                    unsafe { GetOverlappedResult(pipe_handle as _, &mut overlap, &mut bytes, true as _) } != 0;
                if !success || bytes == 0 {
                    log::warn!("GetOverlappedResult failed");
                    break;
                }

                buffer.extend_from_slice(&temp);

                // Print the log
                //
                if success {
                    match std::ffi::CString::from_vec_with_nul(buffer[..bytes as _].to_vec()) {
                        Ok(data) => log::info!("[pipe] {:?}", data),
                        Err(error) => log::error!("Failed to convert buffer to string ({:?})", error),
                    }

                    buffer.clear();
                }
            }

            //     CancelIoEx(pipe->handle, &pipe->overlap);
            //     SetEvent(pipe->ready_event);
            //     WaitForSingleObject(pipe->thread, INFINITE);
            //     CloseHandle(pipe->thread);
        });

        Some(Self {
            handle: pipe_handle,
            _thread: thread_handle,
            thread_running: thread_running_copy,
        })
    }
}

impl Drop for NamedPipe {
    fn drop(&mut self) {
        self.thread_running.store(false, Ordering::Relaxed);

        if self.handle != 0 {
            log::trace!("Dropping the named pipe");
            unsafe { CloseHandle(self.handle as _) };
        }
    }
}
