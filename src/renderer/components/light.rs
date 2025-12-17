use std::sync::{Arc, RwLock};

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Buffer, BufferBinding, Device, ShaderStages,
};

use crate::renderer::{components::transform::Transform, geometry::BindGroupProvider};

#[derive(Debug, Clone)]
pub struct LightSource {
    pub transform: Arc<RwLock<Transform>>,
    color: Vec3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GpuLightSource {
    transform: Transform,
    color: Vec3,
    _padding_2: f32,
}

impl LightSource {
    pub fn new(transform: Arc<RwLock<Transform>>, color: Vec3) -> LightSource {
        Self { transform, color }
    }

    pub fn update_color(&mut self, color: Vec3) {
        self.color = color;
    }

    pub fn to_gpu(&self) -> GpuLightSource {
        GpuLightSource {
            transform: *self.transform.read().unwrap(),
            color: self.color,
            _padding_2: 0.0,
        }
    }
}

impl BindGroupProvider for LightSource {
    fn bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Light Source"),
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

    fn bind_group(
        device: &Device,
        buffer: &Buffer,
        bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Light Bind Group"),
            layout: bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        })
    }
}
