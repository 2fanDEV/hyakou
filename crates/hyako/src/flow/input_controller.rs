use anyhow::Result;
use hyakou_core::{
    Shared, SharedAccess,
    geometry::ray::{Ray, ray_from_screen},
    types::mouse_delta::{MouseAction, MouseButton, MouseDelta, MousePosition, MouseState},
};
use log::{debug, error};
use strum::IntoDiscriminant;
use strum_macros::EnumDiscriminants;
use winit::{
    keyboard::KeyCode,
    window::{CursorGrabMode, Window},
};

use crate::{
    flow::{FlowCommandSender, RendererCommand},
    renderer::{
        SceneRenderer,
        handlers::{InputEvent, keyboard_handler::KeyboardHandler, mouse_handler::MouseHandler},
    },
};

const CLICK_DRAG_THRESHOLD: f32 = 25.0;
#[derive(EnumDiscriminants)]
enum PointerInteraction {
    None,
    PendingClick { start: MousePosition },
    Dragging { start: MousePosition },
}

pub struct InputController {
    _commands: FlowCommandSender,
    keyboard_handler: KeyboardHandler,
    mouse_handler: MouseHandler,
    mouse_delta: MouseDelta,
    pointer_interaction: PointerInteraction,
}

impl InputController {
    pub fn new(commands: FlowCommandSender) -> Self {
        Self {
            _commands: commands,
            keyboard_handler: KeyboardHandler::new(),
            mouse_handler: MouseHandler::new(),
            mouse_delta: MouseDelta::default(),
            pointer_interaction: PointerInteraction::None,
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
        renderer_slot: &Shared<Option<SceneRenderer>>,
        key: KeyCode,
        pressed: bool,
    ) {
        let events = self.keyboard_handler.handle_key(key, pressed);
        let _ = renderer_slot.try_write_shared(|renderer_slot| {
            let Some(renderer) = renderer_slot.as_mut() else {
                return;
            };

            for input_event in events {
                self.handle_input_event(renderer, input_event);
            }
        });
    }

    fn calculate_mouse_position_distance(current: &MousePosition, delta: &MousePosition) -> f32 {
        let dx = delta.x() - current.x();
        let dy = delta.y() - current.y();
        dx.hypot(dy) as f32
    }

    pub fn handle_mouse_motion(
        &mut self,
        renderer_slot: &Shared<Option<SceneRenderer>>,
        dx: f64,
        dy: f64,
        dt: f32,
    ) {
        self.mouse_delta.delta_position =
            hyakou_core::types::mouse_delta::MovementDelta::new(dx, dy);

        match &self.pointer_interaction {
            PointerInteraction::PendingClick { start } => {
                let distance =
                    Self::calculate_mouse_position_distance(start, &self.mouse_delta.position);
                debug!("{:?}", distance);
                if distance > CLICK_DRAG_THRESHOLD {
                    self.pointer_interaction = PointerInteraction::Dragging {
                        start: start.clone(),
                    };
                    self.enqueue_events(renderer_slot, MouseButton::Left, true);
                }
            }
            _ => {}
        }

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
        renderer_slot: &Shared<Option<SceneRenderer>>,
        window: Option<&Window>,
        button: MouseButton,
        pressed: bool,
    ) -> Result<()> {
        self.mouse_delta.state = MouseState::new(
            button,
            if pressed {
                self.pointer_interaction = PointerInteraction::PendingClick {
                    start: self.mouse_delta.position.clone(),
                };
                MouseAction::Clicked
            } else {
                match &self.pointer_interaction {
                    PointerInteraction::None => {}
                    PointerInteraction::PendingClick { start } => {
                        let ray = self.create_ray(renderer_slot)?;
                        self._commands.send(RendererCommand::RayCast { ray });
                    }
                    PointerInteraction::Dragging { start } => {
                        self.enqueue_events(renderer_slot, button, pressed);
                    }
                };
                self.pointer_interaction = PointerInteraction::None;
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

        if !button.eq(&MouseButton::Left) {
            self.enqueue_events(renderer_slot, button, pressed);
        }

        Ok(())
    }

    fn create_ray(
        &self,
        renderer_slot: &std::sync::Arc<
            parking_lot::lock_api::RwLock<parking_lot::RawRwLock, Option<SceneRenderer>>,
        >,
    ) -> Result<Ray> {
        let ray = match renderer_slot.read_shared(|slot| {
            let scene_renderer = slot.as_ref().unwrap();
            let x = self.mouse_delta.position.x() as f32;
            let y = self.mouse_delta.position.y() as f32;
            let size = scene_renderer.ctx.size;
            ray_from_screen(&scene_renderer.camera, x, y, &size)
        }) {
            Ok(rnd) => rnd,
            Err(e) => return Err(e),
        };
        Ok(ray)
    }

    fn enqueue_events(
        &mut self,
        renderer_slot: &std::sync::Arc<
            parking_lot::lock_api::RwLock<parking_lot::RawRwLock, Option<SceneRenderer>>,
        >,
        button: MouseButton,
        pressed: bool,
    ) {
        let events = self.mouse_handler.handle_button(button, pressed);
        let _ = renderer_slot.try_write_shared(|renderer_slot| {
            let Some(renderer) = renderer_slot.as_mut() else {
                return;
            };
            for input_event in events {
                self.handle_input_event(renderer, input_event);
            }
        });
    }

    fn handle_input_event(&self, renderer: &mut SceneRenderer, event: InputEvent) {
        debug!("{:?}", event);
        debug!("{:?}", self.pointer_interaction.discriminant());
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
