use std::{io::Result, sync::Arc};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use log::{debug, error};
use parking_lot::RwLock;
use wgpu::web_sys::HtmlCanvasElement;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowAttributesExtWebSys;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, ElementState, WindowEvent},
    window::{CursorGrabMode, Window, WindowAttributes},
};

use crate::renderer::{
    Renderer,
    handlers::{InputEvent, keyboard_handler::KeyboardHandler, mouse_handler::MouseHandler},
    types::{
        DeltaTime64,
        mouse_delta::{
            MouseAction, MouseButton, MouseDelta, MousePosition, MouseState, MovementDelta,
        },
    },
};

pub struct AppState {
    window: Option<Arc<Window>>,
    html_canvas_element: Option<HtmlCanvasElement>,
    renderer: Arc<RwLock<Option<Renderer>>>,
    keyboard_handler: KeyboardHandler,
    mouse_handler: MouseHandler,
    last_frame_time: Instant,
    mouse_delta: MouseDelta,
}

impl AppState {
    const MIN_TIME_IN_SECONDS: f64 = 0.05;

    pub fn new() -> Result<Self> {
        Ok(Self {
            window: None,
            html_canvas_element: None,
            renderer: Arc::new(RwLock::new(None)),
            keyboard_handler: KeyboardHandler::new(),
            mouse_handler: MouseHandler::new(),
            last_frame_time: Instant::now(),
            mouse_delta: MouseDelta::default(),
        })
    }

    pub fn from_canvas_ref(canvas_ref: HtmlCanvasElement) -> Result<Self> {
        Ok(Self {
            window: None,
            html_canvas_element: Some(canvas_ref),
            renderer: Arc::new(RwLock::new(None)),
            keyboard_handler: KeyboardHandler::new(),
            mouse_handler: MouseHandler::new(),
            last_frame_time: Instant::now(),
            mouse_delta: MouseDelta::default(),
        })
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
        #[cfg(not(target_arch = "wasm32"))]
        let window_attributes =
            WindowAttributes::default().with_inner_size(PhysicalSize::new(1920, 1080));

        #[cfg(target_arch = "wasm32")]
        let window_attributes =
            WindowAttributes::default().with_canvas(self.html_canvas_element.clone());

        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                debug!("{:?}", e);
                panic!();
            }
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            let renderer = pollster::block_on(Renderer::new(window.clone())).unwrap();
            *self.renderer.write() = Some(renderer);
        }

        #[cfg(target_arch = "wasm32")]
        {
            let renderer_slot = self.renderer.clone();
            let window_for_renderer = window.clone();
            spawn_local(async move {
                match Renderer::new(window_for_renderer.clone()).await {
                    Ok(renderer) => {
                        *renderer_slot.write() = Some(renderer);
                        debug!("Renderer initialized for wasm");
                        window_for_renderer.request_redraw();
                    }
                    Err(e) => {
                        error!("Failed to initialize renderer for wasm: {e:?}");
                    }
                }
            });
        }
        self.window = Some(window);
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, _event: ()) {}

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                let delta = self.get_and_update_last_frame_time();
                let mut renderer_guard = self.renderer.write();
                let Some(renderer) = renderer_guard.as_mut() else {
                    return;
                };
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
                let mut renderer_guard = self.renderer.write();
                let Some(renderer) = renderer_guard.as_mut() else {
                    return;
                };
                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(key_code) => {
                        let is_pressed = event.state == ElementState::Pressed;
                        let events = self.keyboard_handler.handle_key(key_code, is_pressed);
                        for event in events {
                            match event {
                                InputEvent::ActionStarted(action) => {
                                    renderer.camera_controller.handle_action(&action, true);
                                }
                                InputEvent::ActionEnded(action) => {
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
        let mut renderer_guard = self.renderer.write();
        let Some(renderer) = renderer_guard.as_mut() else {
            return;
        };
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
                    let mouse_button = match button {
                        0 => MouseButton::Left,
                        1 => MouseButton::Right,
                        2 => MouseButton::Middle,
                        _ => MouseButton::Left,
                    };
                    let is_pressed = state == ElementState::Pressed;

                    self.mouse_delta.state = MouseState::new(
                        mouse_button,
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

                    let events = self.mouse_handler.handle_button(mouse_button, is_pressed);
                    for event in events {
                        match event {
                            InputEvent::ActionStarted(action) => {
                                renderer.camera_controller.handle_action(&action, true);
                            }
                            InputEvent::ActionEnded(action) => {
                                renderer.camera_controller.handle_action(&action, false);
                            }
                        }
                    }
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
        AppState::new().unwrap()
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
