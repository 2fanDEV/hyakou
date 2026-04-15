use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, VertexBufferLayout};

pub trait BufferLayoutProvider {
    fn vertex_buffer_layout() -> VertexBufferLayout<'static>;
}

pub trait BindGroupProvider {
    fn bind_group_layout(device: &Device) -> BindGroupLayout;
    fn bind_group(
        device: &Device,
        buffer: &Buffer,
        bind_group_layout: &BindGroupLayout,
    ) -> BindGroup;
}
