use wgpu::{BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Buffer, BufferBinding, Sampler, ShaderStages, TextureView};

use crate::renderer::geometry::{BindGroupProvider, BufferLayoutProvider, vertices::Vertex};

#[repr(C)]
#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Mesh {
        Self {
            vertices,
            indices
        }
    }

    pub fn bind_group_entries<'a>(buffer: &'a Buffer, texture_view: &'a TextureView, sampler: &'a Sampler) -> Vec<BindGroupEntry<'a>> {
        vec![
            BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(texture_view) },
            BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler)},
            BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Buffer(BufferBinding { buffer: buffer
                , offset: 0, size: None  })}
        ]
    }
}


impl BufferLayoutProvider for Mesh {
    fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        Vertex::vertex_buffer_layout()
    }
}

impl BindGroupProvider for Mesh {
    fn bind_group_layout() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Mesh BindGroupLayoutDescriptor"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        }
    }
}

