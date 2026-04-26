use hyakou_core::{
    Shared, SharedAccess,
    types::mouse_delta::{MouseAction, MouseButton, MouseDelta, MousePosition, MouseState},
};
use log::error;
use winit::{
    keyboard::KeyCode,
    window::{CursorGrabMode, Window},
};

use crate::renderer::{
    Renderer,
    handlers::{InputEvent, keyboard_handler::KeyboardHandler, mouse_handler::MouseHandler},
};

pub struct InputController {
    keyboard_handler: KeyboardHandler,
    mouse_handler: MouseHandler,
    mouse_delta: MouseDelta,
}

impl InputController {
    pub fn new() -> Self {
        Self {
            keyboard_handler: KeyboardHandler::new(),
            mouse_handler: MouseHandler::new(),
            mouse_delta: MouseDelta::default(),
        }
    }

    pub fn handle_cursor_in_window(&mut self, is_inside: bool) {
        self.mouse_delta.set_is_mouse_on_window(is_inside);
    }

    pub fn handle_cursor_moved(&mut self, x: f64, y: f64) {
        self.mouse_delta.position = MousePosition::new(x, y);
    }

    pub fn handle_keyboard_input(
        &mut self,
        renderer_slot: &Shared<Option<Renderer>>,
        key: KeyCode,
        pressed: bool,
    ) {
        let events = self.keyboard_handler.handle_key(key, pressed);
        let _ = renderer_slot.try_write_shared(|renderer_slot| {
            let Some(renderer) = renderer_slot.as_mut() else {
                return;
            };

            for input_event in events {
                Self::handle_input_event(renderer, input_event);
            }
        });
    }

    pub fn handle_mouse_motion(
        &mut self,
        renderer_slot: &Shared<Option<Renderer>>,
        dx: f64,
        dy: f64,
        dt: f32,
    ) {
        self.mouse_delta.delta_position =
            hyakou_core::types::mouse_delta::MovementDelta::new(dx, dy);

        let _ = renderer_slot.try_write_shared(|renderer_slot| {
            let Some(renderer) = renderer_slot.as_mut() else {
                return;
            };

            renderer
                .camera_handler
                .mouse_movement(&mut renderer.camera, &self.mouse_delta, dt);
        });
    }

    pub fn handle_mouse_button(
        &mut self,
        renderer_slot: &Shared<Option<Renderer>>,
        window: Option<&Window>,
        button: MouseButton,
        pressed: bool,
    ) {
        self.mouse_delta.state = MouseState::new(
            button,
            if pressed {
                MouseAction::Clicked
            } else {
                MouseAction::Released
            },
        );

        if let Some(window) = window {
            let grab_mode = CursorGrabMode::None;

            if let Err(cursor_error) = window.set_cursor_grab(grab_mode) {
                error!("Failed to set cursor grab mode: {cursor_error:?}");
            }

            window.set_cursor_visible(!pressed);
        }

        let events = self.mouse_handler.handle_button(button, pressed);
        let _ = renderer_slot.try_write_shared(|renderer_slot| {
            let Some(renderer) = renderer_slot.as_mut() else {
                return;
            };

            for input_event in events {
                Self::handle_input_event(renderer, input_event);
            }
        });
    }

    fn handle_input_event(renderer: &mut Renderer, event: InputEvent) {
        match event {
            InputEvent::ActionStarted(action) => {
                renderer.camera_handler.handle_action(&action, true);
            }
            InputEvent::ActionEnded(action) => {
                renderer.camera_handler.handle_action(&action, false);
            }
        }
    }
}

impl Default for InputController {
    fn default() -> Self {
        Self::new()
    }
}
