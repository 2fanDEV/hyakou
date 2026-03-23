use hyakou_core::{
    components::camera::{camera::Camera, data_structures::CameraMode},
    types::{DeltaTime, mouse_delta::MouseDelta},
};

use crate::renderer::{
    actions::Action,
    handlers::camera::{movement::CameraMovementHandler, state::CameraState},
};

pub mod movement;
pub mod state;

pub struct CameraHandler {
    movement_handler: CameraMovementHandler,
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

    pub fn update(&self, camera: &mut Camera, delta_time: DeltaTime) {
        let updated = match self.state.get_camera_transition(&camera.id) {
            Some(t) => match t.is_active() {
                true => {
                    self.movement_handler
                        .transition_camera_incrementally(camera, t, delta_time);
                    true
                }
                _ => false,
            },
            None => false,
        };
        if !updated {
            self.movement_handler
                .update_camera_with_keyboard(camera, delta_time as f32);
        }
    }
}
