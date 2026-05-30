use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, VertexBufferLayout};

// NOTE: This is a GPU-facing trait in core for now. Move it into hyako when the
// render boundary is finalized during the ECS migration.
pub trait BufferLayoutProvider {
    fn vertex_buffer_layout() -> VertexBufferLayout<'static>;
}

// NOTE: This is a GPU-facing trait in core for now. Move it into hyako when the
// render boundary is finalized during the ECS migration.
pub trait BindGroupProvider {
    fn bind_group_layout(device: &Device) -> BindGroupLayout;
    fn bind_group(
        device: &Device,
        buffer: &Buffer,
        bind_group_layout: &BindGroupLayout,
    ) -> BindGroup;
}
