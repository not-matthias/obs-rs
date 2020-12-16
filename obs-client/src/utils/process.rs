use std::mem::MaybeUninit;
use winapi::um::winuser::{FindWindowA, GetWindowThreadProcessId};

pub fn get_hwnd(window_name: &str) -> Option<usize> {
    let window = unsafe { FindWindowA(std::ptr::null_mut(), format!("{}\0", window_name).as_ptr() as _) };
    if window.is_null() {
        None
    } else {
        Some(window as usize)
    }
}

/// Finds the window thread and process id.
///
/// # Returns
///
/// Returns a tuple: `(process_id, thread_id)`
pub fn get_window_thread_pid(hwnd: usize) -> Option<(u32, u32)> {
    let mut pid = MaybeUninit::uninit();
    let thread_id = unsafe { GetWindowThreadProcessId(hwnd as _, pid.as_mut_ptr()) };

    Some((unsafe { pid.assume_init() }, thread_id))
}
