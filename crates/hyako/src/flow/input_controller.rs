use crate::types::mouse::{MouseAction, MouseButton, MouseDelta, MousePosition, MouseState};
use anyhow::Result;
use hyakou_core::selection::structure::SelectionScope;
use log::{debug, error};
use smallvec::{SmallVec, smallvec};
use strum::IntoDiscriminant;
use strum_macros::EnumDiscriminants;
use winit::{
    keyboard::KeyCode,
    window::{CursorGrabMode, Window},
};

use crate::{
    flow::{FlowCommand, FlowCommandSender},
    renderer::handlers::{
        InputEvent, keyboard_handler::KeyboardHandler, mouse_handler::MouseHandler,
    },
};

const CLICK_DRAG_THRESHOLD: f32 = 25.0;
#[derive(EnumDiscriminants)]
enum PointerInteraction {
    None,
    PendingClick { start: MousePosition },
    Dragging,
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
        key: KeyCode,
        pressed: bool,
    ) -> SmallVec<[InputEvent; 4]> {
        self.keyboard_handler.handle_key(key, pressed)
    }

    fn calculate_mouse_position_distance(current: &MousePosition, delta: &MousePosition) -> f32 {
        let dx = delta.x() - current.x();
        let dy = delta.y() - current.y();
        dx.hypot(dy) as f32
    }

    pub fn handle_mouse_motion(&mut self, dx: f64, dy: f64) -> SmallVec<[InputEvent; 4]> {
        self.mouse_delta.delta_position = crate::types::mouse::MovementDelta::new(dx, dy);
        let mut events = smallvec![];

        match &self.pointer_interaction {
            PointerInteraction::PendingClick { start } => {
                let distance =
                    Self::calculate_mouse_position_distance(start, &self.mouse_delta.position);
                debug!("{:?}", distance);
                if distance > CLICK_DRAG_THRESHOLD {
                    self.pointer_interaction = PointerInteraction::Dragging;
                    events.extend(self.enqueue_events(MouseButton::Left, true));
                }
            }
            _ => {}
        }

        events
    }

    pub fn handle_mouse_button(
        &mut self,
        window: Option<&Window>,
        button: MouseButton,
        pressed: bool,
    ) -> Result<SmallVec<[InputEvent; 4]>> {
        let mut events = smallvec![];
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
                    PointerInteraction::PendingClick { .. } => {
                        self._commands.send(FlowCommand::SelectAtScreenPoint {
                            x: self.mouse_delta.position.x() as f32,
                            y: self.mouse_delta.position.y() as f32,
                            scope: self.selection_scope(),
                        });
                    }
                    PointerInteraction::Dragging => {
                        events.extend(self.enqueue_events(button, pressed));
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
            events.extend(self.enqueue_events(button, pressed));
        }

        Ok(events)
    }

    pub fn mouse_delta(&self) -> MouseDelta {
        self.mouse_delta.clone()
    }

    fn selection_scope(&self) -> SelectionScope {
        if self.keyboard_handler.is_shift_pressed() {
            SelectionScope::Node
        } else {
            SelectionScope::Object
        }
    }

    fn enqueue_events(&mut self, button: MouseButton, pressed: bool) -> SmallVec<[InputEvent; 4]> {
        let events = self.mouse_handler.handle_button(button, pressed);
        debug!("{:?}", self.pointer_interaction.discriminant());
        events
    }
}
