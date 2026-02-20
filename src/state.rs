use std::{sync::Arc, time::Instant};

use log::debug;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, WindowEvent},
    window::{CursorGrabMode, Window, WindowAttributes},
};

use crate::renderer::{
    Renderer,
    handlers::keyboard_handler::KeyboardHandler,
    types::{
        DeltaTime64,
        mouse_delta::{
            MouseAction, MouseButton, MouseDelta, MousePosition, MouseState, MovementDelta,
        },
    },
};

pub struct AppState {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    keyboard_handler: KeyboardHandler,
    last_frame_time: Instant,
    mouse_delta: MouseDelta,
}

impl AppState {
    const MIN_TIME_IN_SECONDS: f64 = 0.05;

    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            keyboard_handler: KeyboardHandler::new(),
            last_frame_time: Instant::now(),
            mouse_delta: MouseDelta::default(),
        }
    }

    fn get_and_update_last_frame_time(&mut self) -> f64 {
        let now = Instant::now();
        let delta_time = self.get_last_frame_time(now);
        self.last_frame_time = now;
        delta_time
    }

    fn get_last_frame_time(&mut self, now: Instant) -> DeltaTime64 {
        let delta = now.duration_since(self.last_frame_time);
        let delta_time = delta.as_secs_f64().min(Self::MIN_TIME_IN_SECONDS);
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
        match event {
            WindowEvent::RedrawRequested => {
                let delta = self.get_and_update_last_frame_time();
                let renderer = self.renderer.as_mut().unwrap();
                renderer.update(delta);
                renderer.render().unwrap();
            }
            #[allow(unused)]
            WindowEvent::CursorEntered { device_id } => {
                self.mouse_delta.set_is_mouse_on_window(true);
            }
            #[allow(unused)]
            /// We are tracking this for future implementation where this might be needed.
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => self.mouse_delta.position = MousePosition::new(position.x, position.y),
            #[allow(unused)]
            WindowEvent::CursorLeft { device_id } => {
                self.mouse_delta.set_is_mouse_on_window(false);
            }
            WindowEvent::KeyboardInput {
                device_id: _device_id,
                event,
                is_synthetic: _is_synthetic,
            } => {
                let renderer = self.renderer.as_mut().unwrap();
                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(key_code) => {
                        let is_pressed = event.state == ElementState::Pressed;
                        let events = self.keyboard_handler.handle_key(key_code, is_pressed);
                        for event in events {
                            match event {
                                crate::renderer::handlers::keyboard_handler::InputEvent::ActionStarted(action) => {
                                    renderer.camera_controller.handle_action(&action, true);
                                }
                                crate::renderer::handlers::keyboard_handler::InputEvent::ActionEnded(action) => {
                                    renderer.camera_controller.handle_action(&action, false);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let delta_time = self.get_last_frame_time(Instant::now()) as f32;
        let renderer = self.renderer.as_mut().unwrap();
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.mouse_delta.delta_position = MovementDelta::new(delta.0, delta.1);
                if self
                    .mouse_delta
                    .is_mouse_button_clicked_and_on_window(MouseButton::Left)
                {}
                renderer.camera_controller.mouse_movement(
                    &mut renderer.camera,
                    &self.mouse_delta,
                    delta_time,
                )
            }
            DeviceEvent::Button { button, state } => {
                if let Some(window) = self.window.clone() {
                    self.mouse_delta.state = MouseState::new(
                        match button {
                            0 => MouseButton::Left,
                            1 => MouseButton::Right,
                            2 => MouseButton::Middle,
                            _ => MouseButton::Left,
                        },
                        match state {
                            ElementState::Pressed => {
                                if let Err(e) = window.set_cursor_grab(CursorGrabMode::Locked) {
                                    log::error!("External Error: {:?}", e)
                                }
                                window.set_cursor_visible(false);
                                MouseAction::Clicked
                            }
                            ElementState::Released => {
                                if let Err(e) = window.set_cursor_grab(CursorGrabMode::None) {
                                    log::error!("External Error: {:?}", e)
                                }
                                window.set_cursor_visible(true);
                                MouseAction::Released
                            }
                        },
                    );
                }
            }
            _ => {}
        }
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
        let actual = state.get_and_update_last_frame_time();
        assert!(actual > 0.0);
        assert!(actual <= AppState::MIN_TIME_IN_SECONDS);
        sleep(Duration::from_secs(1));
        let actual = state.get_and_update_last_frame_time();
        assert!(actual <= AppState::MIN_TIME_IN_SECONDS);
    }

    #[test]
    fn test_accurate_calculation() {
        let mut state = setup();
        state.get_and_update_last_frame_time();
        sleep(Duration::from_millis(16)); // 1000ms / 60 = 16ms. We have around 16ms for each frame to get 60 fps.
        let second_delta = state.get_and_update_last_frame_time();
        assert!(second_delta >= 0.015 && second_delta <= AppState::MIN_TIME_IN_SECONDS);
    }
}
