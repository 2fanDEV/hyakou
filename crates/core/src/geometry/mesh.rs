use crate::geometry::vertices::Vertex;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Mesh {
    pub name: Option<String>,
    pub material_index: Option<usize>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(
        name: Option<String>,
        material_index: Option<usize>,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
    ) -> Mesh {
        Self {
            name,
            material_index,
            vertices,
            indices,
        }
    }
}
