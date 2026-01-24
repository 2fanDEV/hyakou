use std::collections::HashSet;

use smallvec::{smallvec, SmallVec};
use winit::keyboard::KeyCode;

use crate::renderer::{
    actions::Action,
    handlers::key_bindings::{KeyBinding, KeyBindingMap},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    ActionStarted(Action),
    ActionEnded(Action),
}

#[derive(Debug, Default)]
pub struct KeyboardHandler {
    pressed_keys: HashSet<KeyCode>,
    pressed_modifiers: HashSet<KeyCode>,
    key_bindings: KeyBindingMap,
    current_actions: HashSet<Action>,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            pressed_modifiers: HashSet::new(),
            key_bindings: KeyBindingMap::initialize(),
            current_actions: HashSet::new(),
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, is_pressed: bool) -> SmallVec<[InputEvent; 4]> {
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
                true => {
                    self.pressed_modifiers.insert(key);
                }
                false => {
                    self.pressed_keys.insert(key);
                }
            },
            false => match is_modifier {
                true => {
                    self.pressed_modifiers.remove(&key);
                }
                false => {
                    self.pressed_keys.remove(&key);
                }
            },
        };

        let new_actions_vec = self
            .key_bindings
            .resolve_active_actions(&self.pressed_keys, &self.pressed_modifiers);
        let new_actions_set: HashSet<Action> = new_actions_vec.into_iter().collect();

        let mut events = SmallVec::new();

        for action in &self.current_actions {
            if !new_actions_set.contains(action) {
                events.push(InputEvent::ActionEnded(*action));
            }
        }

        for action in &new_actions_set {
            if !self.current_actions.contains(action) {
                events.push(InputEvent::ActionStarted(*action));
            }
        }

        self.current_actions = new_actions_set;

        events
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

    pub fn get_active_actions(&self) -> SmallVec<[Action; 4]> {
        self.current_actions.iter().cloned().collect()
    }
}
