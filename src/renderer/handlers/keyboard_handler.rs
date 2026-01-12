use std::collections::HashMap;

use smallvec::smallvec;
use winit::{event::ElementState, keyboard::KeyCode};

use crate::renderer::{
    actions::Action,
    handlers::key_bindings::{KeyBinding, KeyBindingMap},
};

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub enum KeyState {
    Pressed,
    #[default]
    Released,
}

impl KeyState {
    pub fn convert(state: ElementState) -> Self {
        match state {
            ElementState::Pressed => KeyState::Pressed,
            ElementState::Released => KeyState::Released,
        }
    }
}

#[derive(Debug, Default)]
pub struct KeyboardHandler {
    key_states: HashMap<KeyCode, KeyState>,
    key_bindings: KeyBindingMap,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self {
            key_states: HashMap::new(),
            key_bindings: KeyBindingMap::initialize(),
        }
    }

    pub fn handle_key_state(&mut self, key: KeyCode, key_state: KeyState) {
        self.key_states.insert(key, key_state);
    }

    pub fn get_pressed_keys(&self) -> (Vec<KeyCode>, Vec<KeyCode>) {
        let (modifiers, keys): (Vec<KeyCode>, Vec<KeyCode>) = self
            .key_states
            .iter()
            .filter(|(_, state)| **state == KeyState::Pressed)
            .map(|(key, _)| *key)
            .partition(|key| {
                matches!(
                    key,
                    KeyCode::AltLeft
                        | KeyCode::AltRight
                        | KeyCode::ControlLeft
                        | KeyCode::ControlRight
                        | KeyCode::ShiftLeft
                        | KeyCode::ShiftRight
                        | KeyCode::SuperLeft
                        | KeyCode::SuperRight
                )
            });
        (modifiers, keys)
    }

    pub fn find_action_for_key(&self, key_code: KeyCode) -> Option<&Action> {
        self.key_bindings
            .get_binding(&KeyBinding::new(smallvec![], smallvec![key_code]))
    }

    pub fn check_key_bindings(&self, key_binding: &KeyBinding) -> Option<&Action> {
        self.key_bindings.get_binding(key_binding)
    }

    pub fn is_pressed(&self, key_code: KeyCode) -> bool {
        self.key_states
            .get(&key_code)
            .map_or(false, |&state| state == KeyState::Pressed)
    }
}
