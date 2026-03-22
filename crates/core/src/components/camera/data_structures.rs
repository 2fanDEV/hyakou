use glam::Vec3;

use crate::types::shared::Coordinates;

#[derive(Debug)]
pub enum CameraMode {
    FLY,
    PAN,
    ORBIT,
}

#[derive(Debug)]
pub struct CameraAxes {
    pub forward: Vec3,
    pub forward_mag: f32,
    pub right: Vec3,
    pub view_up: Vec3,
}

#[derive(Debug)]
pub enum CameraTransition {
    Active(Coordinates),
    InActive,
}
