use std::sync::Arc;

use log::debug;
use winit::{
    application::ApplicationHandler,
    window::{Window, WindowAttributes},
};

use crate::renderer::Renderer;

pub struct AppState {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>
}

impl AppState {
    pub fn new() -> Self {
        Self { window: None, renderer: None }
    }
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = match event_loop.create_window(WindowAttributes::default()) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                debug!("{:?}", e);
                panic!();
            }
        };
        let renderer = pollster::block_on(Renderer::new(window.clone())).unwrap();
        self.window = Some(window);
        self.renderer = Some(renderer)
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
    }
}
