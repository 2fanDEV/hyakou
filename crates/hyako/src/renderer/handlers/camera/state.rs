use std::collections::HashMap;

use hyakou_core::{
    components::camera::{camera::Camera, data_structures::CameraTransition},
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
    pub fn set_camera_transition(&mut self, camera: &mut Camera, coordinates: Coordinates3) {
        self.camera_transition
            .insert(camera.id.clone(), CameraTransition::new(coordinates));
    }

    pub fn get_camera_transition(&self, camera_id: &Id) -> Option<&CameraTransition> {
        self.camera_transition.get(camera_id)
    }
}
