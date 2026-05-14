use bytemuck::{Pod, Zeroable};
use glam::Vec4;
use hyakou_core::types::{ids::UniformBufferId, transform::Transform};
use shared::shared;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BufferBinding, Device, ShaderStages,
};

use crate::gpu::buffers::uniform::UniformBuffer;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct OutlineUniform {
    pub color: Vec4,
    pub thickness: f32,
    _padding: [f32; 3],
}

impl OutlineUniform {
    pub fn new(color: Vec4, thickness: f32) -> Self {
        Self {
            color,
            thickness,
            _padding: [0.0; 3],
        }
    }

    pub fn bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Outline Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    pub fn uniform_buffer(device: &Device, outline: &Self) -> UniformBuffer {
        UniformBuffer::new(
            UniformBufferId::new("Outline Uniform Buffer".to_string()),
            device,
            bytemuck::bytes_of(outline),
            shared(Transform::default()),
        )
    }

    pub fn bind_group(
        device: &Device,
        buffer: &UniformBuffer,
        bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Outline Bind Group"),
            layout: bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        })
    }
}
