use hyako::state::AppState;
use log::debug;
use winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::builder().build().unwrap();
    let mut app_state = AppState::new();
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("wgpu_hal::metal::device", log::LevelFilter::Error)
        .try_init()
        .unwrap();
    match event_loop.run_app(&mut app_state) {
        Ok(_a) => debug!("App exited successfully"),
        Err(e) => {
            debug!("{:?}", e);
            panic!()
        }
    };
}
