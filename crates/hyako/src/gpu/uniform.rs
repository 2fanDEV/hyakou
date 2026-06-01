use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Buffer, BufferBinding, Device, ShaderStages, VertexBufferLayout,
};

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

pub trait GpuUniformMetadata {
    const LAYOUT_LABEL: &'static str;
    const BIND_GROUP_LABEL: &'static str;
    const VISIBILITY: ShaderStages;
}

impl<T> BindGroupProvider for T
where
    T: GpuUniformMetadata + Pod + Zeroable,
{
    fn bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(T::LAYOUT_LABEL),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: T::VISIBILITY,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    fn bind_group(
        device: &Device,
        buffer: &Buffer,
        bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some(T::BIND_GROUP_LABEL),
            layout: bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(BufferBinding {
                    buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        })
    }
}
