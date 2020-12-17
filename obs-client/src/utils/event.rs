use crate::hook_info::EVENT_FLAGS;
use winapi::um::{
    handleapi::CloseHandle,
    synchapi::{OpenEventA, SetEvent},
};

pub struct Event {
    handle: usize,
}

impl Event {
    pub fn open<S: AsRef<str>>(name: S) -> Option<Self> {
        let event = unsafe { OpenEventA(EVENT_FLAGS, false as _, format!("{}\0", name.as_ref()).as_ptr() as _) };

        if event.is_null() {
            return None;
        } else {
            log::trace!("Created the event {:?} = 0x{:x}", name.as_ref(), event as usize);
            Some(Self { handle: event as usize })
        }
    }

    /// Sets the event to the signalled state.
    pub fn signal(&self) -> Option<()> {
        if unsafe { SetEvent(self.handle as _) } == 0 {
            None
        } else {
            Some(())
        }
    }
}

impl Drop for Event {
    fn drop(&mut self) {
        log::trace!("Dropping the event");
        unsafe { CloseHandle(self.handle as _) };
    }
}
