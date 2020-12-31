use graphic_offsets::GraphicOffsets;
use winapi::um::winnt::{EVENT_MODIFY_STATE, SYNCHRONIZE};

pub const EVENT_FLAGS: u32 = EVENT_MODIFY_STATE | SYNCHRONIZE;
pub const MUTEX_FLAGS: u32 = SYNCHRONIZE;

/// Signalled by the graphics-hook both when the hook has been setup (see
/// [`init_hook`](https://github.com/obsproject/obs-studio/blob/d46e8b03c963ba15548cf3e62951a26223749c27/plugins/win-capture/graphics-hook/graphics-hook.c#L252)) or when the capture has been freed (see [`capture_free`](https://github.com/obsproject/obs-studio/blob/master/plugins/win-capture/graphics-hook/graphics-hook.c#L807)).
///
/// It's also used by the game-capture to reuse an already existing hook. See [`attempt_existing_hook`](https://github.com/obsproject/obs-studio/blob/d46e8b03c963ba15548cf3e62951a26223749c27/plugins/win-capture/game-capture.c#L685).
pub const EVENT_CAPTURE_RESTART: &str = "CaptureHook_Restart";
pub const EVENT_CAPTURE_STOP: &str = "CaptureHook_Stop";

/// Used by the graphics-hook to signalize that the hook has been set.
///
/// An example can be found in `graphics-hook.c` in the function [capture_init_shtex](https://github.com/obsproject/obs-studio/blob/d46e8b03c963ba15548cf3e62951a26223749c27/plugins/win-capture/graphics-hook/graphics-hook.c#L538).
pub const EVENT_HOOK_READY: &str = "CaptureHook_HookReady";
pub const EVENT_HOOK_EXIT: &str = "CaptureHook_Exit";
pub const EVENT_HOOK_INIT: &str = "CaptureHook_Initialize";

pub const WINDOW_HOOK_KEEPALIVE: &str = "CaptureHook_KeepAlive";

pub const MUTEX_TEXTURE1: &str = "CaptureHook_TextureMutex1";
pub const MUTEX_TEXTURE2: &str = "CaptureHook_TextureMutex2";

pub const SHMEM_HOOK_INFO: &str = "CaptureHook_HookInfo";
pub const SHMEM_TEXTURE: &str = "CaptureHook_Texture";

pub const PIPE_NAME: &str = "CaptureHook_Pipe";

#[derive(Debug)]
#[repr(C)]
pub struct SharedTextureData {
    pub tex_handle: u32,
}

#[derive(Debug)]
#[repr(C)]
pub enum CaptureType {
    Memory,
    Texture,
}

#[derive(Debug)]
#[repr(C)]
pub struct HookInfo {
    /* hook version */
    pub hook_ver_major: u32,
    pub hook_ver_minor: u32,

    /* capture info */
    pub capture_type: CaptureType,
    pub window: u32,
    pub format: u32,
    pub cx: u32,
    pub cy: u32,
    #[doc(hidden)]
    unused_base_cx: u32,
    #[doc(hidden)]
    unused_base_cy: u32,
    pub pitch: u32,
    pub map_id: u32,
    pub map_size: u32,
    pub flip: bool,

    /* additional options */
    pub frame_interval: u64,
    #[doc(hidden)]
    pub unused_use_scale: bool,
    pub force_shmem: bool,
    pub capture_overlay: bool,

    /* hook addresses */
    pub graphics_offsets: GraphicOffsets,

    #[doc(hidden)]
    reserved: [u32; 128],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sizes() {
        assert_eq!(core::mem::size_of::<SharedTextureData>(), 4);
        assert_eq!(core::mem::size_of::<CaptureType>(), 4);

        assert_eq!(core::mem::size_of::<HookInfo>(), 648);
    }
}
