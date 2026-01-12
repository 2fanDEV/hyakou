use std::collections::{HashMap, HashSet};

use log::debug;
use smallvec::smallvec;
use winit::keyboard::KeyCode;

use crate::renderer::{
    actions::Action,
    handlers::key_bindings::{KeyBinding, KeyBindingMap},
};

#[derive(Debug, Default)]
pub struct KeyboardHandler {
    key_states: HashSet<KeyCode>,
    key_bindings: KeyBindingMap,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self {
            key_states: HashSet::new(),
            key_bindings: KeyBindingMap::initialize(),
        }
    }

    pub fn handle_key_state(&mut self, key: KeyCode, is_pressed: bool) {
        debug!("{:?}", self.key_states);
        if is_pressed {
            self.key_states.insert(key);
        } else {
            self.key_states.remove(&key);
        }
    }

    pub fn get_pressed_keys(&self) -> (Vec<KeyCode>, Vec<KeyCode>) {
        let (modifiers, keys): (Vec<KeyCode>, Vec<KeyCode>) =
            self.key_states.iter().partition(|key| {
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
        self.key_states.get(&key_code).is_some()
    }
}
