use hyakou_core::{
    components::camera::camera::Camera,
    types::{base::Id, shared::Coordinates3},
};
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
pub mod bindings;

#[cfg(not(target_arch = "wasm32"))]
pub mod bindings {}

#[wasm_bindgen]
pub struct CameraDO {
    id: Id,
    pub eye: Coordinates3,
    pub target: Coordinates3,
    pub up: Coordinates3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub speed: f32,
    pub sensitivity: f32,
    pub smoothing_factor: f32,
}

impl CameraDO {
    pub fn from_camera(camera: &Camera) -> CameraDO {
        Self {
            id: camera.id.clone(),
            eye: Coordinates3::from_vec3(camera.eye),
            target: Coordinates3::from_vec3(camera.target),
            up: Coordinates3::from_vec3(camera.up),
            aspect: camera.aspect,
            fovy: camera.fovy,
            znear: camera.znear,
            zfar: camera.zfar,
            speed: camera.speed,
            sensitivity: camera.sensitivity,
            smoothing_factor: camera.smoothing_factor,
        }
    }
}
