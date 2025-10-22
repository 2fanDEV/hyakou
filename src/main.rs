use hyako::state::AppState;
use log::debug;
use winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::builder().build().unwrap();
    let mut app_state = AppState::new();
    env_logger::builder().build();
    match event_loop.run_app(&mut app_state) {
        Ok(a) => debug!("App exited succesfully"),
        Err(e) => {
            debug!("{:?}", e);
            panic!()
         }
    };
}
