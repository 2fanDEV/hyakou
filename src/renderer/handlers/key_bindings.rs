use std::collections::HashMap;

use log::{trace, warn};
use winit::keyboard::KeyCode;

use crate::renderer::actions::{Action, CameraActions};

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct KeyBinding {
    modifiers: Vec<KeyCode>,
    keys: Vec<KeyCode>,
}

impl KeyBinding {
    pub fn new(modifiers: Vec<KeyCode>, keys: Vec<KeyCode>) -> Self {
        Self { modifiers, keys }
    }
}

pub struct KeyBindingMap {
    binding: HashMap<KeyBinding, Action>,
}

impl KeyBindingMap {
    pub fn initialize() -> Self {
        let mut binding = HashMap::new();
        binding.insert(
            KeyBinding {
                modifiers: Vec::new(),
                keys: vec![KeyCode::KeyW],
            },
            Action::Camera(CameraActions::Forwards),
        );
        binding.insert(
            KeyBinding {
                modifiers: Vec::new(),
                keys: vec![KeyCode::KeyS],
            },
            Action::Camera(CameraActions::Backwards),
        );
        binding.insert(
            KeyBinding {
                modifiers: Vec::new(),
                keys: vec![KeyCode::KeyD],
            },
            Action::Camera(CameraActions::Right),
        );
        binding.insert(
            KeyBinding {
                modifiers: Vec::new(),
                keys: vec![KeyCode::KeyA],
            },
            Action::Camera(CameraActions::Left),
        );
        binding.insert(
            KeyBinding {
                modifiers: vec![KeyCode::ShiftLeft],
                keys: vec![KeyCode::KeyW],
            },
            Action::Camera(CameraActions::ForwardsModifier),
        );
        Self { binding }
    }

    pub fn add_binding(&mut self, key_bindings: KeyBinding, action: Action) {
        if self.get_binding(&key_bindings).is_some() {
            warn!("The binding is already in use!");
        } else {
            self.binding.insert(key_bindings, action);
        }
    }

    pub fn change_binding(&mut self, previous_bindings: KeyBinding, new_binding: KeyBinding) {
        if let Some(action) = self.remove_binding(&previous_bindings) {
            self.add_binding(new_binding, action);
        } else {
            trace!("Previous Binding did not exist! Binding new binding to action");
        }
    }

    pub fn get_binding(&self, registered_binding: &KeyBinding) -> Option<&Action> {
        self.binding.get(registered_binding)
    }

    pub fn remove_binding(&mut self, previous_bindings: &KeyBinding) -> Option<Action> {
        self.binding.remove(previous_bindings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_w_key_returns_forwards_action() {
        let binding_map = KeyBindingMap::initialize();
        let key_binding = KeyBinding::new(vec![], vec![KeyCode::KeyW]);

        let action = binding_map.get_binding(&key_binding);

        assert_eq!(action, Some(&Action::Camera(CameraActions::Forwards)));
    }

    #[test]
    fn test_s_key_returns_backwards_action() {
        let binding_map = KeyBindingMap::initialize();
        let key_binding = KeyBinding::new(vec![], vec![KeyCode::KeyS]);

        let action = binding_map.get_binding(&key_binding);

        assert_eq!(action, Some(&Action::Camera(CameraActions::Backwards)));
    }

    #[test]
    fn test_a_key_returns_left_action() {
        let binding_map = KeyBindingMap::initialize();
        let key_binding = KeyBinding::new(vec![], vec![KeyCode::KeyA]);

        let action = binding_map.get_binding(&key_binding);

        assert_eq!(action, Some(&Action::Camera(CameraActions::Left)));
    }

    #[test]
    fn test_d_key_returns_right_action() {
        let binding_map = KeyBindingMap::initialize();
        let key_binding = KeyBinding::new(vec![], vec![KeyCode::KeyD]);

        let action = binding_map.get_binding(&key_binding);

        assert_eq!(action, Some(&Action::Camera(CameraActions::Right)));
    }

    #[test]
    fn test_shift_w_returns_forwards_modifier_action() {
        let binding_map = KeyBindingMap::initialize();
        let key_binding = KeyBinding::new(vec![KeyCode::ShiftLeft], vec![KeyCode::KeyW]);

        let action = binding_map.get_binding(&key_binding);

        assert_eq!(
            action,
            Some(&Action::Camera(CameraActions::ForwardsModifier))
        );
    }

    #[test]
    fn test_non_existent_binding_returns_none() {
        let binding_map = KeyBindingMap::initialize();
        let key_binding = KeyBinding::new(vec![], vec![KeyCode::Space]);

        let action = binding_map.get_binding(&key_binding);

        assert_eq!(action, None);
    }

    #[test]
    fn test_add_binding_creates_new_binding() {
        let mut binding_map = KeyBindingMap::initialize();
        let key_binding = KeyBinding::new(vec![], vec![KeyCode::Space]);

        binding_map.add_binding(key_binding.clone(), Action::Camera(CameraActions::Forwards));

        let action = binding_map.get_binding(&key_binding);
        assert_eq!(action, Some(&Action::Camera(CameraActions::Forwards)));
    }

    #[test]
    fn test_add_binding_conflict_does_not_overwrite() {
        let mut binding_map = KeyBindingMap::initialize();
        let key_binding = KeyBinding::new(vec![], vec![KeyCode::KeyW]);

        binding_map.add_binding(key_binding.clone(), Action::Camera(CameraActions::Backwards));

        let action = binding_map.get_binding(&key_binding);
        assert_eq!(action, Some(&Action::Camera(CameraActions::Forwards)));
    }

    #[test]
    fn test_remove_binding_removes_existing_binding() {
        let mut binding_map = KeyBindingMap::initialize();
        let key_binding = KeyBinding::new(vec![], vec![KeyCode::KeyW]);

        let removed_action = binding_map.remove_binding(&key_binding);

        assert_eq!(removed_action, Some(Action::Camera(CameraActions::Forwards)));
        assert_eq!(binding_map.get_binding(&key_binding), None);
    }

    #[test]
    fn test_remove_non_existent_binding_returns_none() {
        let mut binding_map = KeyBindingMap::initialize();
        let key_binding = KeyBinding::new(vec![], vec![KeyCode::Space]);

        let removed_action = binding_map.remove_binding(&key_binding);

        assert_eq!(removed_action, None);
    }

    #[test]
    fn test_change_binding_moves_action_to_new_key() {
        let mut binding_map = KeyBindingMap::initialize();
        let old_binding = KeyBinding::new(vec![], vec![KeyCode::KeyW]);
        let new_binding = KeyBinding::new(vec![], vec![KeyCode::Space]);

        binding_map.change_binding(old_binding.clone(), new_binding.clone());

        assert_eq!(binding_map.get_binding(&old_binding), None);
        assert_eq!(
            binding_map.get_binding(&new_binding),
            Some(&Action::Camera(CameraActions::Forwards))
        );
    }

    #[test]
    fn test_modifier_without_key_does_not_match() {
        let binding_map = KeyBindingMap::initialize();
        let key_binding = KeyBinding::new(vec![KeyCode::ShiftLeft], vec![KeyCode::KeyS]);

        let action = binding_map.get_binding(&key_binding);

        assert_eq!(action, None);
    }
}
