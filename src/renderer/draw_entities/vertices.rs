use bytemuck::{Pod, Zeroable};
use nalgebra::Vector3;
use wgpu::VertexBufferLayout;

use crate::renderer::draw_entities::BufferLayouts;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pos: Vector3<f32>,
    color: Vector3<f32>,
}

impl Vertex {
    pub const fn new(pos: Vector3<f32>, color: Vector3<f32>) -> Self {
        Self { pos, color }
    }
}

impl BufferLayouts for Vertex {
    fn layouts() -> VertexBufferLayout<'static> {
        const ATTRIBS: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}
