use std::{sync::Arc, time::Instant};

use log::debug;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    window::{Window, WindowAttributes},
};

use crate::renderer::Renderer;

pub struct AppState {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    last_frame_time: Instant,
}

impl AppState {
    const MIN_TIME_IN_SECONDS: f64 = 0.05;

    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            last_frame_time: Instant::now(),
        }
    }

    fn calculate_last_frame_time(&mut self) -> f64 {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_time);
        let mut delta_time = delta.as_secs_f64();
        delta_time = delta_time.min(Self::MIN_TIME_IN_SECONDS);
        self.last_frame_time = now;
        delta_time
    }
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = match event_loop.create_window(
            WindowAttributes::default().with_inner_size(PhysicalSize::new(1920, 1080)),
        ) {
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
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let mut mouse_pos = PhysicalPosition::new(0.0, 0.0);
        match event {
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                mouse_pos = position;
            }
            WindowEvent::RedrawRequested => {
                let delta = self.calculate_last_frame_time();
                self.renderer.as_mut().unwrap().update(delta);
            }
            WindowEvent::KeyboardInput {
                device_id: _device_id,
                event,
                is_synthetic: _is_synthetic,
            } => match event.physical_key {
                winit::keyboard::PhysicalKey::Code(key_code) => {
                    self.renderer
                        .as_mut()
                        .unwrap()
                        .camera_controller
                        .handle_key(key_code, event.state.is_pressed());
                }
                winit::keyboard::PhysicalKey::Unidentified(_) => {}
            },
            _ => {}
        }

        self.renderer.as_mut().unwrap().render(mouse_pos).unwrap();
    }
}

#[cfg(test)]
mod tests {

    use std::{thread::sleep, time::Duration};

    use super::*;

    fn setup() -> AppState {
        AppState::new()
    }

    #[test]
    fn test_clamping_strategy() {
        let mut state = setup();
        let actual = state.calculate_last_frame_time();
        assert!(actual > 0.0);
        assert!(actual <= AppState::MIN_TIME_IN_SECONDS);
        sleep(Duration::from_secs(1));
        let actual = state.calculate_last_frame_time();
        assert!(actual <= AppState::MIN_TIME_IN_SECONDS);
    }

    #[test]
    fn test_accurate_calculation() {
        let mut state = setup();
        state.calculate_last_frame_time();
        sleep(Duration::from_millis(16)); // 1000ms / 60 = 16ms. We have around 16ms for each frame to get 60 fps.
        let second_delta = state.calculate_last_frame_time();
        assert!(second_delta >= 0.015 && second_delta <= AppState::MIN_TIME_IN_SECONDS);
    }
}
