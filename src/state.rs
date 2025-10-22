use log::debug;
use winit::{application::ApplicationHandler, window::{Window, WindowAttributes}};

pub struct AppState {
    window: Option<Window>
}


impl AppState {
    pub fn new() -> Self {
        Self {
            window: None
        }
    }
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = match event_loop.create_window(WindowAttributes::default()) {
            Ok(window) => window,
            Err(e) => {
                debug!("{:?}", e);
                panic!();
            },
        };
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
    }
}