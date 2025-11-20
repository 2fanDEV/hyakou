use std::sync::Arc;

use log::debug;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{KeyEvent, WindowEvent},
    window::{Window, WindowAttributes},
};

use crate::renderer::Renderer;

pub struct AppState {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
        }
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
        let mut mouse_pos = PhysicalPosition::new(0.0, 0.0);
        match event {
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                mouse_pos = position;
            },
            WindowEvent::RedrawRequested => {
                self.renderer.as_mut().unwrap().update();
            }
            _ => {}
        }

        let key_event = match event {
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(key_code) => {
                        self.renderer.as_mut().unwrap().camera_controller.handle_key(key_code, event.state.is_pressed());
                    },
                    winit::keyboard::PhysicalKey::Unidentified(native_key_code) => {},
                }
            }
            _ => {}
        };

        self.renderer.as_mut().unwrap().render(mouse_pos).unwrap();
    }
}
