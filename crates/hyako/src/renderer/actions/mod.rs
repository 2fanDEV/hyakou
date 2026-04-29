pub mod camera_actions;

pub use camera_actions::CameraActions;
use strum_macros::EnumDiscriminants;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumDiscriminants)]
pub enum Action {
    Camera(CameraActions),
}

impl Action {
    pub fn as_camera(&self) -> Option<&CameraActions> {
        match self {
            Action::Camera(action) => Some(action),
        }
    }
}
