use wgpu::VertexBufferLayout;

pub mod vertices;

pub trait BufferLayouts {
    fn layouts() -> VertexBufferLayout<'static>;
}