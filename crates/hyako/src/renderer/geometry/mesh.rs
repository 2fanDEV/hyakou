
use crate::renderer::geometry::vertices::Vertex;

#[repr(C)]
#[derive(Debug)]
pub struct Mesh {
    pub name: Option<String>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>
}

impl Mesh {
    pub fn new(name: Option<String>, vertices: Vec<Vertex>, indices: Vec<u32>) -> Mesh {
        Self {
            name,
            vertices,
            indices
        }
    }
}


