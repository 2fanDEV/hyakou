use std::ops::Deref;

use crate::renderer::{
    components::render_mesh::Transform,
    geometry::{BufferLayoutProvider, mesh::Mesh, vertices::Vertex},
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
