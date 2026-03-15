use std::collections::HashSet;

use smallvec::{SmallVec, smallvec};

use crate::renderer::{
    actions::Action,
    handlers::{
        InputEvent,
        mouse_bindings::{MouseBinding, MouseBindingMap},
    },
};
use core::mouse_delta::MouseButton;

#[derive(Debug, Default)]
pub struct MouseHandler {
    pressed_buttons: HashSet<MouseButton>,
    mouse_bindings: MouseBindingMap,
    current_actions: HashSet<Action>,
}

impl MouseHandler {
    pub fn new() -> Self {
        Self {
            pressed_buttons: HashSet::new(),
            mouse_bindings: MouseBindingMap::initialize(),
            current_actions: HashSet::new(),
        }
    }

    pub fn handle_button(
        &mut self,
        button: MouseButton,
        is_pressed: bool,
    ) -> SmallVec<[InputEvent; 4]> {
        match is_pressed {
            true => {
                self.pressed_buttons.insert(button);
            }
            false => {
                self.pressed_buttons.remove(&button);
            }
        };

        let new_actions_vec = self
            .mouse_bindings
            .resolve_active_actions(&self.pressed_buttons);
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

    pub fn get_pressed_buttons(&self) -> &HashSet<MouseButton> {
        &self.pressed_buttons
    }

    pub fn find_action_for_button(&self, button: MouseButton) -> Option<&Action> {
        self.mouse_bindings
            .get_binding(&MouseBinding::new(smallvec![button]))
    }

    pub fn check_mouse_bindings(&self, mouse_binding: &MouseBinding) -> Option<&Action> {
        self.mouse_bindings.get_binding(mouse_binding)
    }

    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    pub fn get_active_actions(&self) -> SmallVec<[Action; 4]> {
        self.current_actions.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::actions::CameraActions;

    #[test]
    fn test_new_handler_has_no_pressed_buttons() {
        let handler = MouseHandler::new();
        assert!(handler.get_pressed_buttons().is_empty());
    }

    #[test]
    fn test_handle_button_press_adds_to_pressed() {
        let mut handler = MouseHandler::new();
        handler.handle_button(MouseButton::Left, true);

        assert!(handler.is_pressed(MouseButton::Left));
        assert!(handler.get_pressed_buttons().contains(&MouseButton::Left));
    }

    #[test]
    fn test_handle_button_release_removes_from_pressed() {
        let mut handler = MouseHandler::new();
        handler.handle_button(MouseButton::Left, true);
        handler.handle_button(MouseButton::Left, false);

        assert!(!handler.is_pressed(MouseButton::Left));
    }

    #[test]
    fn test_press_left_mouse_generates_drag_action_started() {
        let mut handler = MouseHandler::new();
        let events = handler.handle_button(MouseButton::Left, true);

        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            InputEvent::ActionStarted(Action::Camera(CameraActions::Drag))
        );
    }

    #[test]
    fn test_release_left_mouse_generates_drag_action_ended() {
        let mut handler = MouseHandler::new();
        handler.handle_button(MouseButton::Left, true);

        let events = handler.handle_button(MouseButton::Left, false);

        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            InputEvent::ActionEnded(Action::Camera(CameraActions::Drag))
        );
    }

    #[test]
    fn test_get_active_actions_returns_current_actions() {
        let mut handler = MouseHandler::new();
        handler.handle_button(MouseButton::Left, true);

        let actions = handler.get_active_actions();

        assert!(actions.contains(&Action::Camera(CameraActions::Drag)));
    }

    #[test]
    fn test_get_active_actions_empty_when_no_buttons_pressed() {
        let handler = MouseHandler::new();

        let actions = handler.get_active_actions();

        assert!(actions.is_empty());
    }

    #[test]
    fn test_find_action_for_button() {
        let handler = MouseHandler::new();

        let action = handler.find_action_for_button(MouseButton::Left);

        assert_eq!(action, Some(&Action::Camera(CameraActions::Drag)));
    }

    #[test]
    fn test_find_action_for_unbound_button() {
        let handler = MouseHandler::new();

        let action = handler.find_action_for_button(MouseButton::Right);

        assert_eq!(action, None);
    }
}
