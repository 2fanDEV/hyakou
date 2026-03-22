use hyakou_core::components::camera::data_structures::CameraMode;

use crate::renderer::handlers::camera::{movement::CameraMovementHandler, state::CameraState};

pub mod movement;
pub mod state;

pub struct CameraHandler {
    movement_handler: CameraMovementHandler,
    state: CameraState,
}

impl CameraHandler {
    pub fn new() -> Self {
        Self {
            movement_handler: CameraMovementHandler::new(CameraMode::PAN),
            state: CameraState::new(),
        }
    }
}
