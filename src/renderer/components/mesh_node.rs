use std::ops::Deref;

use log::debug;
use nalgebra::{Matrix4, Perspective3, Point3, Vector3};
use wgpu::{BindGroupEntry, Buffer, BufferBinding, Sampler, ShaderStages, TextureView};

use crate::renderer::geometry::{BindGroupProvider, BufferLayoutProvider, mesh::Mesh, vertices::Vertex};

pub struct MeshNode {
    mesh: Mesh,
    pub model: Matrix4<f32>,
}

impl Deref for MeshNode {
    type Target = Mesh;

    fn deref(&self) -> &Self::Target {
        &self.mesh
    }
}

impl MeshNode {
    pub fn new(mesh: Mesh, m: [[f32; 4]; 4]) -> MeshNode {
        let mesh_node = MeshNode {
            mesh,
            #[rustfmt::skip]
            model: Matrix4::new(
                 m[0][0], m[0][1], m[0][2], m[0][3],
                 m[1][0], m[1][1], m[1][2], m[1][3], 
                 m[2][0], m[2][1], m[2][2], m[2][3], 
                 m[3][0], m[3][1], m[3][2], m[3][3]
                ),
        };
        mesh_node
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

impl BufferLayoutProvider for MeshNode {
    fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        Vertex::vertex_buffer_layout()
    }
}

impl BindGroupProvider for MeshNode {
    fn bind_group_layout() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("MeshNode BindGroupLayoutDescriptor"),
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

