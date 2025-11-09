use crate::renderer::geometry::vertices::Vertex;

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>
}
