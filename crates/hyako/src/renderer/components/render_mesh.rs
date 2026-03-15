use parking_lot::RwLock;
use std::sync::Arc;

use uuid::Uuid;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::renderer::{
    components::{LightType, mesh_node::MeshNode, model_matrix::ModelMatrixUniform},
    geometry::BindGroupProvider,
    util::Concatable,
};

use hyakou_core::{
    ModelMatrixBindingMode,
    ids::{MeshId, UniformBufferId},
    transform::Transform,
    uniform::UniformBuffer,
};

#[derive(Debug, Clone)]
pub struct RenderMesh {
    pub id: MeshId,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub light_type: LightType,
    pub transform: Arc<RwLock<Transform>>,
    pub model_uniform_buffer: Option<UniformBuffer>,
    pub model_bind_group: Option<BindGroup>,
}

impl RenderMesh {
    pub fn new(
        device: &Device,
        mesh_node: MeshNode,
        light_type: &LightType,
        label: Option<MeshId>,
        model_binding_mode: ModelMatrixBindingMode,
        model_bind_group_layout: Option<&BindGroupLayout>,
    ) -> Self {
        let id = label.unwrap_or(MeshId(Uuid::new_v4().to_string()));
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

        let transform = Arc::new(RwLock::new(mesh_node.transform));
        let (model_uniform_buffer, model_bind_group) = Self::create_model_binding_resources(
            device,
            &id,
            transform.clone(),
            model_binding_mode,
            model_bind_group_layout,
        );

        Self {
            id,
            vertex_buffer,
            index_buffer,
            light_type: light_type.clone(),
            index_count: mesh_node.indices.len() as u32,
            transform,
            model_uniform_buffer,
            model_bind_group,
        }
    }

    fn create_model_binding_resources(
        device: &Device,
        id: &MeshId,
        transform: Arc<RwLock<Transform>>,
        model_binding_mode: ModelMatrixBindingMode,
        model_bind_group_layout: Option<&BindGroupLayout>,
    ) -> (Option<UniformBuffer>, Option<BindGroup>) {
        if model_binding_mode != ModelMatrixBindingMode::Uniform {
            return (None, None);
        }

        let bind_group_layout = model_bind_group_layout.expect(
            "Uniform model binding mode requires a model bind group layout in RenderMesh::new",
        );
        let model_uniform = ModelMatrixUniform::new(transform.read().get_matrix());
        let uniform_buffer = UniformBuffer::new(
            UniformBufferId::new(format!("Model Matrix Buffer: {}", id.0)),
            device,
            bytemuck::bytes_of(&model_uniform),
            transform,
        );
        let bind_group = ModelMatrixUniform::bind_group(device, &uniform_buffer, bind_group_layout);

        (Some(uniform_buffer), Some(bind_group))
    }
}
