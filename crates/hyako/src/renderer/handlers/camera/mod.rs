use hyakou_core::{
    components::camera::{camera::Camera, data_structures::{CameraMode, CameraModeIter}},
    types::{DeltaTime, mouse_delta::MouseDelta},
};

use crate::renderer::{
    actions::Action,
    handlers::camera::{movement::CameraMovementHandler, state::CameraState},
};

pub mod movement;
pub mod state;

pub struct CameraModeHandler {
   camera_mode: CameraModeIter,
}

impl CameraModeHandler {
   pub fn set_camera_mode(self, mode: CameraMode) {

   }
}

pub struct CameraHandler {
    movement_handler: CameraMovementHandler,
    pub camera_mode_handler: CameraModeHandler,
    pub state: CameraState,
}

impl CameraHandler {
    pub fn new() -> Self {
        Self {
            movement_handler: CameraMovementHandler::new(CameraMode::PAN),
            state: CameraState::new(),
        }
    }

    pub fn mouse_movement(
        &mut self,
        camera: &mut Camera,
        mouse_delta: &MouseDelta,
        delta_time: DeltaTime,
    ) {
        self.movement_handler
            .mouse_movement(camera, mouse_delta, delta_time);
    }

    pub fn handle_action(&mut self, action: &Action, is_pressed: bool) {
        self.movement_handler.handle_action(action, is_pressed);
    }

    pub fn update(&mut self, camera: &mut Camera, delta_time: DeltaTime) {
        let updated = match self.state.get_camera_transition_mut(&camera.id) {
            Some(transition) if transition.is_active() => {
                self.movement_handler
                    .transition_camera_incrementally(camera, transition, delta_time);
                true
            }
            _ => false,
        };

        if !updated {
            self.movement_handler
                .update_camera_with_keyboard(camera, delta_time as f32);
        }
    }

    pub fn
}
