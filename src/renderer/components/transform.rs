use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct Transform {
    pub position: Vec3,
    _padding1: f32,
    pub rotation: Quat,
    pub scale: Vec3,
    _padding2: f32,
}

impl Transform {
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Transform {
        Self {
            position,
            rotation,
            scale,
            ..Default::default()
        }
    }

    pub fn translate(&mut self, delta: Vec3) {
        self.position += delta;
    }

    pub fn rotation(&mut self, delta: Quat) {
        self.rotation *= delta;
    }
    pub fn scale(&mut self, delta: Vec3) {
        self.scale += delta;
    }

    pub fn get_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.position)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_scale(self.scale)
    }
}
