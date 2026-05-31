use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3, Vec4};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, Default)]
pub struct Vertex {
    pub position: Vec3,
    pub tex_coords: Vec2,
    pub normals: Vec3,
    pub colors: Vec4,
}

impl Vertex {
    pub fn new(position: Vec3, tex_coords: Vec2, normals: Vec3, colors: Vec4) -> Self {
        Self {
            position,
            tex_coords,
            colors,
            normals,
        }
    }
}
