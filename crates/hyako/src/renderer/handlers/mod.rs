pub mod asset_handler;
pub mod camera;
pub mod key_bindings;
pub mod keyboard_handler;
pub mod mouse_bindings;
pub mod mouse_handler;
pub mod resource_handler;

use crate::renderer::actions::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    ActionStarted(Action),
    ActionEnded(Action),
}
