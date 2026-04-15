use std::ops::Deref;

use crate::{
    geometry::{mesh::Mesh, vertices::Vertex},
    traits::BufferLayoutProvider,
    types::transform::Transform,
};

pub struct MeshNode {
    mesh: Mesh,
    pub transform: Transform,
}

impl Deref for MeshNode {
    type Target = Mesh;

    fn deref(&self) -> &Self::Target {
        &self.mesh
    }
}

impl MeshNode {
    pub fn new(mesh: Mesh, transform: Transform) -> MeshNode {
        let mesh_node = MeshNode { mesh, transform };
        mesh_node
    }
}

impl BufferLayoutProvider for MeshNode {
    fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        Vertex::vertex_buffer_layout()
    }
}
