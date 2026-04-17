use hyakou_core::{
    components::camera::{camera::Camera, data_structures::CameraMode},
    types::{DeltaTime, mouse_delta::MouseDelta},
};

use crate::renderer::{
    actions::Action,
    handlers::camera::{
        mode::CameraModeHandler, movement::CameraMovementHandler, state::CameraState,
    },
};

pub mod mode;
pub mod movement;
pub mod state;

pub struct CameraHandler {
    movement_handler: CameraMovementHandler,
    pub camera_mode_handler: CameraModeHandler,
    pub state: CameraState,
}

impl CameraHandler {
    pub fn new(mode: CameraMode) -> Self {
        let camera_mode_handler = CameraModeHandler::new(mode);
        Self {
            camera_mode_handler,
            movement_handler: CameraMovementHandler::new(),
            state: CameraState::new(),
        }
    }

    pub fn set_mode(&mut self, mode: CameraMode) {
        self.camera_mode_handler.set(mode);
    }

    pub fn mode(&self) -> &CameraMode {
        self.camera_mode_handler.mode()
    }

    pub fn mouse_movement(
        &mut self,
        camera: &mut Camera,
        mouse_delta: &MouseDelta,
        delta_time: DeltaTime,
    ) {
        self.movement_handler.mouse_movement(
            camera,
            self.camera_mode_handler.mode(),
            mouse_delta,
            delta_time,
        );
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
            self.movement_handler.update_camera_with_keyboard(
                camera,
                self.camera_mode_handler.mode(),
                delta_time as f32,
            );
        }
    }
}
