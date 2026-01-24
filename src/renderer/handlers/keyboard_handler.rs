use std::collections::HashSet;

use smallvec::{SmallVec, smallvec};
use winit::keyboard::KeyCode;

use crate::renderer::{
    actions::Action,
    handlers::key_bindings::{KeyBinding, KeyBindingMap},
};

#[derive(Debug, Default)]
pub struct KeyboardHandler {
    pressed_keys: HashSet<KeyCode>,
    pressed_modifiers: HashSet<KeyCode>,
    key_bindings: KeyBindingMap,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            pressed_modifiers: HashSet::new(),
            key_bindings: KeyBindingMap::initialize(),
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, is_pressed: bool) {
        let is_modifier = matches!(
            key,
            KeyCode::AltLeft
                | KeyCode::AltRight
                | KeyCode::ControlLeft
                | KeyCode::ControlRight
                | KeyCode::ShiftLeft
                | KeyCode::ShiftRight
                | KeyCode::SuperLeft
                | KeyCode::SuperRight
        );

        match is_pressed {
            true => match is_modifier {
                true => self.pressed_modifiers.insert(key),
                false => self.pressed_keys.insert(key),
            },
            false => match is_modifier {
                true => self.pressed_modifiers.remove(&key),
                false => self.pressed_keys.remove(&key),
            },
        };
    }

    pub fn get_pressed_keys(&self) -> &HashSet<KeyCode> {
        &self.pressed_keys
    }

    pub fn get_pressed_modifiers(&self) -> &HashSet<KeyCode> {
        &self.pressed_modifiers
    }

    pub fn find_action_for_key(&self, key_code: KeyCode) -> Option<&Action> {
        self.key_bindings
            .get_binding(&KeyBinding::new(smallvec![], smallvec![key_code]))
    }

    pub fn find_action_for_modifier(&self, modifier: KeyCode) -> Option<&Action> {
        self.key_bindings
            .get_binding(&KeyBinding::new(smallvec![modifier], smallvec![]))
    }

    pub fn find_action_for_modifiers(&self, modifiers: SmallVec<[KeyCode; 5]>) -> Option<&Action> {
        self.key_bindings
            .get_binding(&KeyBinding::new(modifiers, smallvec![]))
    }

    pub fn find_action_for_keybinding(
        &self,
        modifiers: SmallVec<[KeyCode; 5]>,
        keys: SmallVec<[KeyCode; 5]>,
    ) -> Option<&Action> {
        self.key_bindings
            .get_binding(&KeyBinding::new(modifiers, keys))
    }

    pub fn check_key_bindings(&self, key_binding: &KeyBinding) -> Option<&Action> {
        self.key_bindings.get_binding(key_binding)
    }

    pub fn is_pressed(&self, key_code: KeyCode) -> bool {
        self.pressed_keys.get(&key_code).is_some()
    }

    pub fn get_active_actions(&self, changed_key: Option<KeyCode>) -> SmallVec<[&Action; 4]> {
        let mut actions: SmallVec<[&Action; 4]> = SmallVec::new();
        let pressed_keys = self.pressed_keys.into_iter().collect::<Vec<_>>();
        let pressed_modifiers = self.pressed_modifiers.into_iter().collect::<Vec<_>>();
        let full_binding_action = self.key_bindings.get_binding(&KeyBinding::new(
            pressed_modifiers.into(),
            pressed_keys.into(),
        ));

        actions
    }
}
