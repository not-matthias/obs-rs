pub struct CaptureConfig {
    frames: u32,
}

/// Everything needed to run the game capture.
pub struct Context {}

pub struct Capture {}

impl Capture {
    pub fn new<S: AsRef<str>>(window_name: S) -> Self {
        let _ = window_name;

        Self {}
    }

    pub fn capture_frame<T>(&mut self) -> Option<(Vec<T>, (u32, u32))> {
        None
    }
}
