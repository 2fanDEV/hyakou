use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3, Vec4};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Device, Sampler, ShaderStages, TextureSampleType,
    TextureView, TextureViewDimension, VertexBufferLayout,
};

use crate::renderer::geometry::{BindGroupProvider, BufferLayoutProvider};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, Default)]
pub struct Vertex {
    pub position: Vec3,
    pub tex_coords: Vec2,
    pub normals: Vec3,
    pub colors: Vec4,
}

impl Vertex {
    pub fn new(position: Vec3, tex_coords: Vec2, normals: Vec3, colors: Vec4) -> Self {
        Self {
            position,
            tex_coords,
            colors,
            normals,
            ..Default::default()
        }
    }

    pub fn create_bind_group(
        device: &Device,
        texture_view: &TextureView,
        sampler: &Sampler,
    ) -> (BindGroupLayout, BindGroup) {
        let layout = Self::bind_group_layout(device);
        (
            layout.clone(),
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("Vertex Bind Group"),
                layout: &layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            }),
        )
    }
}

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

impl BindGroupProvider for Vertex {
    fn bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    #[allow(unused)]
    fn bind_group(
        device: &Device,
        buffer: &wgpu::Buffer,
        bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        todo!()
    }
}
