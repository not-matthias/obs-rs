# obs-rs
Capture frames of any game using OBS. 

## Features

This projects uses the [`graphics-hook`](https://github.com/obsproject/obs-studio/tree/master/plugins/win-capture/graphics-hook) implementation from the [obs-studio](https://github.com/obsproject/obs-studio) project, to capture frames of any game. 

- The graphics hook is signed and whitelisted by all anti-cheats as it's used by streamers and content creators as well. 
- Works for many graphics APIs (D3D9, D3D10, D3D11, Vulkan, ...) and thus also for many different games.
- This implementation is **extremely fast**, because it only copies the pixels from the framebuffer. On my machine, this crate is almost **5 times faster** compared to an implementation using [`BitBlt`](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-bitblt). 

## Example

```rust
use obs_client::Capture;

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        .init()
        .unwrap();

    let mut capture = Capture::new("Rainbow Six");
    if capture.try_launch().is_err() {
        println!("Failed to launch the capture");
        return;
    }

    loop {
        let _ = capture.capture_frame::<u8>();
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
```

