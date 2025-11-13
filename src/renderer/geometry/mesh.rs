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
}


