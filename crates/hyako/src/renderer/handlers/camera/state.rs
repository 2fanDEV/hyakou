use std::collections::HashMap;

use hyakou_core::{
    components::camera::{
        camera::Camera,
        data_structures::{CameraAnimationRequest, CameraAnimationStateSnapshot, CameraTransition},
    },
    types::{base::Id, shared::Coordinates3},
};

#[derive(Debug)]
pub struct CameraState {
    pub camera_transition: HashMap<Id, CameraTransition>,
}

impl CameraState {
    pub fn new() -> Self {
        Self {
            camera_transition: HashMap::new(),
        }
    }

    pub fn animate_camera(&mut self, camera: &Camera, request: CameraAnimationRequest) {
        self.camera_transition.insert(
            camera.id.clone(),
            CameraTransition::new(Coordinates3::from_vec3(camera.eye), request, camera.speed),
        );
    }

    pub fn get_camera_transition_mut(&mut self, camera_id: &Id) -> Option<&mut CameraTransition> {
        self.camera_transition.get_mut(camera_id)
    }

    pub fn stop_camera_animation(&mut self, camera_id: &Id) {
        if let Some(transition) = self.camera_transition.get_mut(camera_id) {
            transition.stop();
        }
    }

    pub fn camera_animation_state(&self, camera: &Camera) -> CameraAnimationStateSnapshot {
        match self.camera_transition.get(&camera.id) {
            Some(transition) => transition.state_snapshot(),
            None => CameraAnimationStateSnapshot::inactive(Coordinates3::from_vec3(camera.eye)),
        }
    }
}
