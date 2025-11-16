use std::ops::Deref;

use nalgebra::Matrix4;

use crate::renderer::geometry::{BufferLayoutProvider, mesh::Mesh, vertices::Vertex};

pub struct MeshNode {
    mesh: Mesh,
    pub model_matrix: Matrix4<f32>,
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
            model_matrix: Matrix4::new(
                 m[0][0], m[0][1], m[0][2], m[0][3],
                 m[1][0], m[1][1], m[1][2], m[1][3], 
                 m[2][0], m[2][1], m[2][2], m[2][3], 
                 m[3][0], m[3][1], m[3][2], m[3][3]
                ),
        };
        mesh_node
    }

}

impl BufferLayoutProvider for MeshNode {
    fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        Vertex::vertex_buffer_layout()
    }
}