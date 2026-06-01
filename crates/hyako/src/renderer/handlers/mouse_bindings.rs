use std::collections::HashSet;

use smallvec::{SmallVec, smallvec};

use crate::renderer::{
    actions::{Action, CameraActions},
    handlers::binding_map::BindingMap,
};
use crate::types::mouse::MouseButton;

const MAX_MOUSE_BUTTON_BIND_COUNT: usize = 5;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct MouseBinding {
    buttons: SmallVec<[MouseButton; MAX_MOUSE_BUTTON_BIND_COUNT]>,
}

impl MouseBinding {
    pub fn new(mut buttons: SmallVec<[MouseButton; MAX_MOUSE_BUTTON_BIND_COUNT]>) -> Self {
        buttons.sort();
        Self { buttons }
    }
}

#[derive(Debug, Default)]
pub struct MouseBindingMap {
    bindings: BindingMap<MouseBinding>,
}

impl MouseBindingMap {
    pub fn initialize() -> Self {
        Self {
            bindings: BindingMap::from_entries([(
                MouseBinding::new(smallvec![MouseButton::Left]),
                Action::Camera(CameraActions::Drag),
            )]),
        }
    }

    pub fn add_binding(&mut self, mouse_binding: MouseBinding, action: Action) {
        self.bindings.add_binding(mouse_binding, action);
    }

    pub fn change_binding(&mut self, previous_bindings: MouseBinding, new_binding: MouseBinding) {
        self.bindings.change_binding(previous_bindings, new_binding);
    }

    pub fn get_binding(&self, registered_binding: &MouseBinding) -> Option<&Action> {
        self.bindings.get_binding(registered_binding)
    }

    pub fn remove_binding(&mut self, previous_bindings: &MouseBinding) -> Option<Action> {
        self.bindings.remove_binding(previous_bindings)
    }

    pub fn resolve_active_actions(&self, pressed_buttons: &HashSet<MouseButton>) -> Vec<Action> {
        let mut active_actions = Vec::new();
        for (binding, action) in self.bindings.iter() {
            if binding
                .buttons
                .iter()
                .all(|btn| pressed_buttons.contains(btn))
            {
                active_actions.push(*action);
            }
        }
        active_actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_left_mouse_returns_drag_action() {
        let binding_map = MouseBindingMap::initialize();
        let mouse_binding = MouseBinding::new(smallvec![MouseButton::Left]);
        let action = binding_map.get_binding(&mouse_binding);
        assert_eq!(action, Some(&Action::Camera(CameraActions::Drag)));
    }

    #[test]
    fn test_resolve_active_actions_with_left_mouse_pressed() {
        let binding_map = MouseBindingMap::initialize();
        let mut pressed_buttons = HashSet::new();
        pressed_buttons.insert(MouseButton::Left);
        let actions = binding_map.resolve_active_actions(&pressed_buttons);
        assert!(actions.contains(&Action::Camera(CameraActions::Drag)));
    }

    #[test]
    fn test_resolve_active_actions_with_no_buttons_pressed() {
        let binding_map = MouseBindingMap::initialize();
        let pressed_buttons = HashSet::new();
        let actions = binding_map.resolve_active_actions(&pressed_buttons);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_non_existent_binding_returns_none() {
        let binding_map = MouseBindingMap::initialize();
        let mouse_binding = MouseBinding::new(smallvec![MouseButton::Right]);
        let action = binding_map.get_binding(&mouse_binding);
        assert_eq!(action, None);
    }

    #[test]
    fn test_add_binding_creates_new_binding() {
        let mut binding_map = MouseBindingMap::initialize();
        let mouse_binding = MouseBinding::new(smallvec![MouseButton::Right]);
        binding_map.add_binding(mouse_binding.clone(), Action::Camera(CameraActions::Drag));
        let action = binding_map.get_binding(&mouse_binding);
        assert_eq!(action, Some(&Action::Camera(CameraActions::Drag)));
    }

    #[test]
    fn test_remove_binding_removes_existing_binding() {
        let mut binding_map = MouseBindingMap::initialize();
        let mouse_binding = MouseBinding::new(smallvec![MouseButton::Left]);
        let removed_action = binding_map.remove_binding(&mouse_binding);
        assert_eq!(removed_action, Some(Action::Camera(CameraActions::Drag)));
        assert_eq!(binding_map.get_binding(&mouse_binding), None);
    }

    #[test]
    fn test_remove_non_existent_binding_returns_none() {
        let mut binding_map = MouseBindingMap::initialize();
        let mouse_binding = MouseBinding::new(smallvec![MouseButton::Middle]);
        let removed_action = binding_map.remove_binding(&mouse_binding);
        assert_eq!(removed_action, None);
    }

    #[test]
    fn test_multi_button_binding() {
        let mut binding_map = MouseBindingMap::initialize();
        let multi_binding = MouseBinding::new(smallvec![MouseButton::Left, MouseButton::Right]);
        binding_map.add_binding(
            multi_binding.clone(),
            Action::Camera(CameraActions::SpeedModifier),
        );
        let action = binding_map.get_binding(&multi_binding);
        assert_eq!(action, Some(&Action::Camera(CameraActions::SpeedModifier)));
    }

    #[test]
    fn test_multi_button_binding_requires_all_buttons() {
        let mut binding_map = MouseBindingMap::initialize();
        let multi_binding = MouseBinding::new(smallvec![MouseButton::Left, MouseButton::Right]);
        binding_map.add_binding(multi_binding, Action::Camera(CameraActions::SpeedModifier));
        let mut pressed_buttons = HashSet::new();
        pressed_buttons.insert(MouseButton::Left);
        let actions = binding_map.resolve_active_actions(&pressed_buttons);
        assert!(!actions.contains(&Action::Camera(CameraActions::SpeedModifier)));
        pressed_buttons.insert(MouseButton::Right);
        let actions = binding_map.resolve_active_actions(&pressed_buttons);
        assert!(actions.contains(&Action::Camera(CameraActions::SpeedModifier)));
    }
}
