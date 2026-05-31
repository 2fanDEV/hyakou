use hyakou_core::components::mesh_node::MeshNode;

use crate::gpu::traits::BufferLayoutProvider;

impl BufferLayoutProvider for MeshNode {
    fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        hyakou_core::geometry::vertices::Vertex::vertex_buffer_layout()
    }
}
