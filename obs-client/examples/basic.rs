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

    let mut fps = fps_counter::FPSCounter::new();
    loop {
        let (buffer, (width, height)) = capture.capture_frame::<u8>().unwrap();
        println!("{:?} | {:?}x{:?} | {:?}", fps.tick(), width, height, buffer.len());
    }
}
