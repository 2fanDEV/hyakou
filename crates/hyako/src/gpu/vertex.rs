use hyakou_core::geometry::vertices::Vertex;
use wgpu::VertexBufferLayout;

use crate::gpu::uniform::BufferLayoutProvider;

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
