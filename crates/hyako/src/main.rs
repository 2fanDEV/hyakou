use hyako::state::AppState;
use log::debug;
use winit::{event_loop::EventLoop, platform::pump_events::EventLoopExtPumpEvents};

fn main() {
    #[cfg(any(target_family = "unix", target_family = "windows"))]
    start_app_os();
}

fn start_app_wasm() {}

fn start_app_os() {
    let event_loop = EventLoop::builder().build().unwrap();
    let mut app_state = AppState::new();
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("wgpu_hal::metal::device", log::LevelFilter::Error)
        .filter_module("naga", log::LevelFilter::Error)
        .try_init()
        .unwrap();
    match event_loop.run_app(&mut app_state) {
        Ok(_) => debug!("App exited successfully"),
        Err(e) => {
            debug!("{:?}", e);
            panic!()
        }
    };
}
