use obs_client::Capture;
use std::ffi::{c_char, CStr};

#[no_mangle]
pub extern "C" fn create_capture(name_str: *const c_char) -> *mut Capture {
    let name = unsafe { CStr::from_ptr(name_str).to_string_lossy().into_owned() };

    let capture = Capture::new(name.as_str());
    Box::into_raw(Box::new(capture))
}

#[no_mangle]
pub extern "C" fn free_capture(capture: *mut Capture) {
    if capture.is_null() {
        return;
    }
    let capture = unsafe { Box::from_raw(capture) };
    core::mem::drop(capture);
}

#[no_mangle]
pub extern "C" fn try_launch_capture(capture: *mut Capture) -> bool {
    if capture.is_null() {
        return false;
    }

    if let Err(e) = unsafe { (*capture).try_launch() } {
        eprintln!("Failed to launch capture: {:?}", e);
        false
    } else {
        true
    }
}

#[repr(C)]
pub struct Frame {
    width: usize,
    height: usize,
    data: *mut u8,
}

#[no_mangle]
pub extern "C" fn capture_frame(capture: *mut Capture) -> *mut Frame {
    if capture.is_null() {
        return std::ptr::null_mut();
    }

    let frame = unsafe { (*capture).capture_frame() };
    let (data, (width, height)) = match frame {
        Ok(frame) => frame,
        Err(e) => {
            eprintln!("Failed to capture frame: {:?}", e);
            return std::ptr::null_mut();
        }
    };

    let frame = Frame {
        width,
        height,
        data: data.as_mut_ptr(),
    };
    Box::into_raw(Box::new(frame))
}
