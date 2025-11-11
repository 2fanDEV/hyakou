use nalgebra::Matrix4;
use wgpu::{
    BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, TextureView,
    VertexBufferLayout,
};

pub mod mesh;
pub mod render_object;
pub mod vertices;
pub trait BufferLayoutProvider {
    fn vertex_buffer_layout() -> VertexBufferLayout<'static>;
}

pub trait BindGroupProvider {
    fn bind_group_layout() -> BindGroupLayoutDescriptor<'static>;
}