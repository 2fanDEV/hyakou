use ::shared::{Shared, SharedAccess};
use bevy_ecs::component::Component;
use bytemuck::{Pod, Zeroable};
use glam::Vec3;

use crate::types::transform::Transform;

#[derive(Component, Debug, Clone)]
pub struct LightSource {
    pub transform: Shared<Transform>,
    color: Vec3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
// NOTE: This is GPU-facing data in core for now. Move it into hyako when the
// render boundary is finalized during the ECS migration.
pub struct GpuLightSource {
    transform: Transform,
    color: Vec3,
    _padding_2: f32,
}

impl LightSource {
    pub fn new(transform: Shared<Transform>, color: Vec3) -> LightSource {
        Self { transform, color }
    }

    pub fn update_color(&mut self, color: Vec3) {
        self.color = color;
    }

    pub fn to_gpu(&self) -> Option<GpuLightSource> {
        self.transform
            .try_read_shared(|t| t.clone())
            .map(|t| GpuLightSource {
                transform: t,
                color: self.color,
                _padding_2: 0.0,
            })
            .ok()
    }
}
