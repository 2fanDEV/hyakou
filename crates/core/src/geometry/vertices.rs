use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3, Vec4};
use wgpu::VertexBufferLayout;

use crate::traits::BufferLayoutProvider;

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

impl BufferLayoutProvider for Vertex {
    fn vertex_buffer_layout() -> VertexBufferLayout<'static> {
        const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3, 3 => Float32x4];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}
