use std::ops::Deref;

use crate::{
    geometry::{mesh::Mesh, node::NodeMetadata, vertices::Vertex},
    traits::BufferLayoutProvider,
    types::transform::Transform,
};

pub struct MeshNode {
    mesh: Mesh,
    pub transform: Transform,
    pub node_metadata: NodeMetadata,
}

impl Deref for MeshNode {
    type Target = Mesh;

    fn deref(&self) -> &Self::Target {
        &self.mesh
    }
}

impl MeshNode {
    pub fn new(mesh: Mesh, transform: Transform, node_metadata: NodeMetadata) -> MeshNode {
        MeshNode {
            mesh,
            transform,
            node_metadata,
        }
    }
}

impl BufferLayoutProvider for MeshNode {
    fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        Vertex::vertex_buffer_layout()
    }
}
