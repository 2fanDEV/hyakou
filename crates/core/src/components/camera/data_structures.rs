use glam::Vec3;

use crate::types::shared::Coordinates3;

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
pub struct CameraTransition {
    target_coords: Coordinates3,
    status: TransitionStatus,
    increments: f32,
}

impl CameraTransition {
    pub fn new(target_coords: Coordinates3) -> Self {
        Self {
            target_coords,
            status: TransitionStatus::Active,
            increments: 20.0,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, TransitionStatus::Active)
    }

    pub fn target_coords(&self) -> &Coordinates3 {
        &self.target_coords
    }

    pub fn increments(&self) -> f32 {
        self.increments
    }
}

#[derive(Debug)]
pub enum TransitionStatus {
    Active,
    InActive,
}
