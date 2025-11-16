use bytemuck::{Pod, Zeroable};
use nalgebra::Vector3;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Buffer, BufferBinding, Device, ShaderStages};

use crate::renderer::geometry::BindGroupProvider;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LightSource {
    position: Vector3<f32>,
    color: Vector3<f32>
}

impl LightSource {
  pub fn new(position: Vector3<f32>, color: Vector3<f32>) -> LightSource {
    Self {
        position,
        color
    }
  }

  pub fn bind_group(device: &Device, buffer: &Buffer) -> (BindGroupLayout, BindGroup)  {
        let bind_group_layout = &Self::bind_group_layout(device);
        (bind_group_layout.clone(), 
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Light Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(
                    BufferBinding { buffer: &buffer, offset: 0, size: None }
                ) 
            }],
        }))
  }
}

impl BindGroupProvider for LightSource {
    fn bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(
        &BindGroupLayoutDescriptor {
            label: Some("Light Source"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                }
            ],
        })
    }
}