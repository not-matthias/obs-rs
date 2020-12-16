// TODO: Implement this properly

use winapi::um::{
    handleapi::INVALID_HANDLE_VALUE,
    winbase::{
        CreateNamedPipeA, FILE_FLAG_OVERLAPPED, PIPE_ACCESS_DUPLEX, PIPE_READMODE_MESSAGE, PIPE_TYPE_MESSAGE, PIPE_WAIT,
    },
};

pub const IPC_PIPE_BUFFER_SIZE: u32 = 1024;

pub fn create_pipe(name: String) -> Option<()> {
    let name = format!("\\\\.\\pipe\\{}\0", name);
    let handle = unsafe {
        CreateNamedPipeA(
            name.as_ptr() as _,
            PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED,
            PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
            1,
            IPC_PIPE_BUFFER_SIZE,
            IPC_PIPE_BUFFER_SIZE,
            0,
            std::ptr::null_mut(),
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        None
    } else {
        Some(())
    }
}
