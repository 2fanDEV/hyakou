use std::sync::{Arc, RwLock};

use uuid::Uuid;
use wgpu::{
    Buffer, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::renderer::{
    components::{LightType, mesh_node::MeshNode, transform::Transform},
    util::Concatable,
};

#[derive(Debug, Clone)]
pub struct RenderMesh {
    pub id: String,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub light_type: LightType,
    pub transform: Arc<RwLock<Transform>>,
}

impl RenderMesh {
    pub fn new(
        device: &Device,
        mesh_node: MeshNode,
        light_type: &LightType,
        label: Option<String>,
    ) -> Self {
        let id = label.unwrap_or(Uuid::new_v4().to_string());
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer: ".to_string().concat(&id)),
            contents: bytemuck::cast_slice(&mesh_node.vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer: ".to_string().concat(&id)),
            contents: bytemuck::cast_slice(&mesh_node.indices),
            usage: BufferUsages::INDEX,
        });
        Self {
            id,
            vertex_buffer,
            index_buffer,
            light_type: light_type.clone(),
            index_count: mesh_node.indices.len() as u32,
            transform: Arc::new(RwLock::new(mesh_node.transform)),
        }
    }
}
