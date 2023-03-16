use crate::{
    error::ObsError,
    hook_info::{
        HookInfo, SharedTextureData, EVENT_CAPTURE_RESTART, EVENT_CAPTURE_STOP, EVENT_HOOK_EXIT, EVENT_HOOK_INIT,
        EVENT_HOOK_READY, PIPE_NAME, SHMEM_HOOK_INFO, SHMEM_TEXTURE, WINDOW_HOOK_KEEPALIVE,
    },
    utils::{color::BGRA8, d3d11, event::Event, file_mapping::FileMapping, mutex::Mutex, pipe::NamedPipe},
};
use std::{mem, ptr, slice};
use winapi::{
    shared::{
        dxgi::{IDXGISurface1, DXGI_MAPPED_RECT, DXGI_MAP_READ, DXGI_RESOURCE_PRIORITY_MAXIMUM},
        winerror::FAILED,
    },
    um::{
        d3d11::{
            ID3D11Device, ID3D11DeviceContext, ID3D11Resource, ID3D11Texture2D, D3D11_CPU_ACCESS_READ,
            D3D11_USAGE_STAGING,
        },
        errhandlingapi::GetLastError,
    },
};
use wio::com::ComPtr;

pub mod error;
pub mod hook_info;
pub mod utils;

#[derive(Default)]
pub struct CaptureConfig {
    window_name: String,
    #[allow(dead_code)]
    frames: u32,
    capture_overlays: bool,
}

/// Everything needed to run the game capture.
#[allow(dead_code)]
#[derive(Default)]
pub struct Context {
    hwnd: usize,
    pid: u32,
    thread_id: u32,
    texture_handle: u32,

    keepalive_mutex: Option<Mutex>,
    pipe: Option<NamedPipe>,

    hook_restart: Option<Event>,
    hook_stop: Option<Event>,
    hook_init: Option<Event>,
    hook_ready: Option<Event>,
    hook_exit: Option<Event>,

    device: Option<ComPtr<ID3D11Device>>,
    device_context: Option<ComPtr<ID3D11DeviceContext>>,
    resource: Option<ComPtr<ID3D11Resource>>,

    // Temporary storage so that we don't leak memory / have UB.
    frame_surface: Option<ComPtr<IDXGISurface1>>,
}

pub struct Capture {
    config: CaptureConfig,
    context: Context,
}

unsafe impl Send for Capture {}
unsafe impl Sync for Capture {}

impl Capture {
    pub fn new<S: ToString>(window_name: S) -> Self {
        Self {
            config: CaptureConfig {
                window_name: window_name.to_string(),
                ..Default::default()
            },
            context: Context::default(),
        }
    }

    fn init_keepalive(&mut self) -> Result<(), ObsError> {
        log::info!("Initializing the keepalive mutex");

        if self.context.keepalive_mutex.is_none() {
            let name = format!("{}{}", WINDOW_HOOK_KEEPALIVE, self.context.pid);
            self.context.keepalive_mutex = Some(Mutex::create(name).ok_or(ObsError::CreateMutex)?);
        }

        Ok(())
    }

    fn init_pipe(&mut self) -> Result<(), ObsError> {
        if self.context.pipe.is_none() {
            let name = format!("{}{}", PIPE_NAME, self.context.pid);
            self.context.pipe = Some(NamedPipe::create(name).ok_or(ObsError::CreatePipe)?);
        }

        Ok(())
    }

    fn attempt_existing_hook(&mut self) -> bool {
        log::info!("Attempting to reuse the existing hook");

        // Create the event if not yet done
        //
        if let Some(event) = Event::open(format!("{}{}", EVENT_CAPTURE_RESTART, self.context.pid)) {
            log::info!("Found an existing hook. Signalling the event");

            if event.signal().is_none() {
                log::warn!("Failed to signal the event");
            };

            true
        } else {
            log::info!("Found no existing hook.");
            false
        }
    }

    fn init_hook_info(&mut self) -> Result<(), ObsError> {
        log::info!("Initializing the hook information");

        let mut hook_info = FileMapping::<HookInfo>::open(format!("{}{}", SHMEM_HOOK_INFO, self.context.pid))
            .ok_or_else(|| ObsError::CreateFileMapping(unsafe { GetLastError() as u32 }))?;

        let graphic_offsets = graphic_offsets::load_graphic_offsets().map_err(ObsError::LoadGraphicOffsets)?;
        unsafe { (**hook_info).graphics_offsets = graphic_offsets };
        unsafe { (**hook_info).capture_overlay = self.config.capture_overlays };
        unsafe { (**hook_info).force_shmem = false };
        unsafe { (**hook_info).unused_use_scale = false };

        log::info!("Hook info: {:?}", unsafe { &**hook_info });

        Ok(())
    }

    fn init_events(&mut self) -> Result<(), ObsError> {
        macro_rules! open_event {
            ($var_name:tt, $event_name:expr) => {
                // if self.context.$var_name.is_none() {
                if let Some(event) = Event::open(format!("{}{}", $event_name, self.context.pid)) {
                    self.context.$var_name = Some(event)
                } else {
                    log::warn!("Couldn't find {:?} ({:?}).", $event_name, unsafe {
                        GetLastError()
                    });
                    return Err(ObsError::CreateEvent);
                }
                // }
            };
        }

        open_event!(hook_restart, EVENT_CAPTURE_RESTART);
        open_event!(hook_stop, EVENT_CAPTURE_STOP);
        open_event!(hook_init, EVENT_HOOK_INIT);
        open_event!(hook_ready, EVENT_HOOK_READY);
        open_event!(hook_exit, EVENT_HOOK_EXIT);

        Ok(())
    }

    /// Tries to launch the capture.
    pub fn try_launch(&mut self) -> Result<(), ObsError> {
        let hwnd = utils::process::get_hwnd(&*self.config.window_name).ok_or(ObsError::ProcessNotFound)?;
        let (pid, thread_id) = utils::process::get_window_thread_pid(hwnd).ok_or(ObsError::ProcessNotFound)?;

        log::info!(
            "Found the process. pid = {}, thread id = {}, hwnd = {}",
            pid,
            thread_id,
            hwnd
        );

        self.context.hwnd = hwnd;
        self.context.pid = pid;
        self.context.thread_id = thread_id;

        self.init_keepalive()?;
        self.init_pipe()?;

        if !self.attempt_existing_hook() {
            log::info!(
                "Trying to inject the graphics hook into the thread {}.",
                self.context.thread_id
            );
            inject_helper::inject_graphics_hook(self.context.thread_id, true).map_err(ObsError::Inject)?;
        }

        self.init_hook_info()?;
        self.init_events()?;

        // Create and signal the hook init event
        //
        let event = Event::open(format!("{}{}", EVENT_HOOK_INIT, self.context.pid)).ok_or(ObsError::CreateEvent)?;
        if event.signal().is_none() {
            log::warn!("Failed to signal the hook init event");
        };

        // Extract the handle
        //
        let hook_info = FileMapping::<HookInfo>::open(format!("{}{}", SHMEM_HOOK_INFO, self.context.pid))
            .ok_or_else(|| ObsError::CreateFileMapping(unsafe { GetLastError() as u32 }))?;

        let texture_data = FileMapping::<SharedTextureData>::open(format!(
            "{}_{}_{}",
            SHMEM_TEXTURE,
            unsafe { (**hook_info).window },
            unsafe { (**hook_info).map_id }
        ))
        .ok_or_else(|| ObsError::CreateFileMapping(unsafe { GetLastError() as u32 }))?;

        let texture_handle = unsafe { (**texture_data).tex_handle };
        self.context.texture_handle = texture_handle;

        // Initialize the d3d11 variables
        //

        let (device, device_context) = d3d11::create_device()?;
        let resource = d3d11::open_resource(&device, self.context.texture_handle)?;

        self.context.device = Some(device);
        self.context.device_context = Some(device_context);
        self.context.resource = Some(resource);

        Ok(())
    }

    fn map_resource(&mut self) -> Result<(DXGI_MAPPED_RECT, (usize, usize)), ObsError> {
        // Cleanup resources from the previous run
        //
        if let Some(frame_surface) = &self.context.frame_surface {
            unsafe { frame_surface.Unmap() };
            self.context.frame_surface = None;
        }

        // Copy the resource (https://github.com/bryal/dxgcap-rs/blob/master/src/lib.rs#L187)
        //
        let frame_texture = self
            .context
            .resource
            .as_ref()
            .unwrap()
            .cast::<ID3D11Texture2D>()
            .unwrap();
        let mut texture_desc = unsafe {
            let mut texture_desc = mem::zeroed();
            frame_texture.GetDesc(&mut texture_desc);
            texture_desc
        };

        // Configure the description to make the texture readable
        texture_desc.Usage = D3D11_USAGE_STAGING;
        texture_desc.BindFlags = 0;
        texture_desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ;
        texture_desc.MiscFlags = 0;

        log::trace!("Creating a 2d texture");
        let readable_texture = unsafe {
            let mut readable_texture = ptr::null_mut();
            let hr = self.context.device.as_ref().unwrap().CreateTexture2D(
                &texture_desc,
                ptr::null(),
                &mut readable_texture,
            );
            if FAILED(hr) {
                log::error!("Failed to create the 2d texture {:x}", hr);
                return Err(ObsError::CreateTexture);
            }
            ComPtr::from_raw(readable_texture)
        };

        // Lower priorities causes stuff to be needlessly copied from gpu to ram,
        // causing huge ram usage on some systems.
        unsafe { readable_texture.SetEvictionPriority(DXGI_RESOURCE_PRIORITY_MAXIMUM) };
        let readable_surface = readable_texture.up::<ID3D11Resource>();

        log::trace!("Copying the resources");
        unsafe {
            self.context
                .device_context
                .as_ref()
                .unwrap()
                .CopyResource(readable_surface.as_raw(), frame_texture.up::<ID3D11Resource>().as_raw());
        }
        let frame_surface: ComPtr<IDXGISurface1> = readable_surface.cast().unwrap();
        log::trace!("Texture Size: {} x {}", texture_desc.Width, texture_desc.Height);

        // Resource to Surface (https://github.com/bryal/dxgcap-rs/blob/master/src/lib.rs#L229)
        //
        log::trace!("Mapping the surface");
        let mapped_surface = unsafe {
            let mut mapped_surface = mem::zeroed();
            let result = frame_surface.Map(&mut mapped_surface, DXGI_MAP_READ);
            if FAILED(result) {
                log::error!("Failed to map surface: {:x}", result);
                frame_surface.Release();
                return Err(ObsError::MapSurface);
            }

            mapped_surface
        };

        // Set the frame surface so that we can unmap it in the next run. We have to do
        // it this way so that we can don't have to copy the pixels to a new buffer.
        //
        self.context.frame_surface = Some(frame_surface);

        Ok((
            mapped_surface,
            (texture_desc.Width as usize, texture_desc.Height as usize),
        ))
    }

    /// Captures the frame and returns it.
    ///
    /// # Returns
    ///
    /// Returns a tuple with the:
    /// - Frame
    /// - Width and Height
    pub fn capture_frame<T>(&mut self) -> Result<(&mut [T], (usize, usize)), ObsError> {
        // Restart the capture if the game has been rehooked
        //
        if self
            .context
            .hook_restart
            .as_ref()
            .map(|e| e.signalled().is_some())
            .unwrap_or(true)
        {
            log::warn!("The restart event has been signalled. Restarting the capture.");
            self.try_launch()?;
        }

        // Copy the texture
        //
        let (mapped_surface, (width, height)) = self.map_resource()?;

        let byte_size = |x| x * mem::size_of::<BGRA8>() / mem::size_of::<T>();
        let stride = mapped_surface.Pitch as usize / mem::size_of::<BGRA8>();
        let mapped_pixels =
            unsafe { slice::from_raw_parts_mut(mapped_surface.pBits as *mut T, byte_size(stride) * height) };

        Ok((mapped_pixels, (width, height)))
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        if let Some(frame_surface) = &self.context.frame_surface {
            unsafe { frame_surface.Unmap() };
        }
    }
}
