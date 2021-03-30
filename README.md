# obs-rs
Capture frames of any game using OBS. 

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

