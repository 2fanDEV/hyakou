pub mod camera_actions;

pub use camera_actions::CameraActions;
use strum_macros::EnumDiscriminants;

use crate::renderer::actions::camera_actions::CameraHandlerAction;

pub trait ActionImpl {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumDiscriminants)]
pub enum Action {
    Camera(CameraActions),
    CameraHandler(CameraHandlerAction),
}
