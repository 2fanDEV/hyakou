use winit::{event::ElementState, keyboard::KeyCode};

use crate::renderer::util::keycode_to_index;

pub struct Keybinding {}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub enum KeyState {
    PRESSED,
    #[default]
    RELEASED,
}

impl KeyState {
    pub fn convert(state: ElementState) -> Self {
        match state {
            ElementState::Pressed => KeyState::PRESSED,
            ElementState::Released => KeyState::RELEASED,
        }
    }
}

pub struct KeyboardHandler {
    key_states: [KeyState; 256],
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self {
            key_states: [KeyState::RELEASED; 256],
        }
    }

    pub fn handle_key_state(&mut self, key: KeyCode, key_state: KeyState) {
        self.key_states[keycode_to_index(key)] = key_state
    }

    pub fn is_pressed(&self, key_code: KeyCode) -> bool {
        self.key_states[keycode_to_index(key_code)].eq(&KeyState::PRESSED)
    }
}
