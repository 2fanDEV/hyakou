use nalgebra::{Matrix4, Quaternion, UnitQuaternion, Vector3, Vector4};
use uuid::Uuid;
use wgpu::{
    Buffer, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::renderer::{
    components::{LightType, mesh_node::MeshNode},
    util::Concatable,
};

#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
}

#[derive(Debug, Clone)]
pub struct RenderMesh {
    pub id: String,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub light_type: LightType,
    pub transform: Transform,
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
            transform: mesh_node.transform,
        }
    }

    pub fn get_matrix(&self) -> Matrix4<f32> {
        self.transform.position * self.transform.rotation.to_homogeneous() * self.transform.scale
    }

    pub fn calculate_matrix(transform: Transform) -> Matrix4<f32> {
        Matrix4::
    }
}
