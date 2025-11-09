use wgpu::{BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, TextureView, VertexBufferLayout};

pub mod vertices;
pub mod mesh;

pub trait BufferLayoutProvider {
    fn vertex_buffer_layout() -> VertexBufferLayout<'static>;
}

pub trait BindGroupProvider {
    fn bind_group_layout() -> BindGroupLayoutDescriptor<'static>;
}