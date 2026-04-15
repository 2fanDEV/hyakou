use hyako::state::AppState;
use hyakou_core::events::Event;
use log::debug;
use winit::event_loop::EventLoop;

#[allow(unused)]
fn main() {
    let mut app_state = AppState::new().unwrap();

    #[cfg(any(target_family = "unix", target_family = "windows"))]
    start_app_os(&mut app_state);
}

#[allow(unused)]
fn start_app_os(app_state: &mut AppState) {
    let event_loop = EventLoop::<Event>::with_user_event().build().unwrap();
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("wgpu_hal::metal::device", log::LevelFilter::Error)
        .filter_module("naga", log::LevelFilter::Error)
        .try_init()
        .unwrap();
    match event_loop.run_app(app_state) {
        Ok(_) => debug!("App exited successfully"),
        Err(e) => {
            debug!("{:?}", e);
            panic!()
        }
    };
}
