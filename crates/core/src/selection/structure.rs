use crate::types::ids::MeshId;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SelectionScope {
    Node,
    Object,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectionTarget {
    mesh_id: MeshId,
    outline_mesh_ids: Vec<MeshId>,
    scope: SelectionScope,
}

impl SelectionTarget {
    pub fn new(mesh_id: MeshId, outline_mesh_ids: Vec<MeshId>, scope: SelectionScope) -> Self {
        Self {
            mesh_id,
            outline_mesh_ids,
            scope,
        }
    }

    pub fn mesh_id(&self) -> &MeshId {
        &self.mesh_id
    }

    pub fn outline_mesh_ids(&self) -> &Vec<MeshId> {
        &self.outline_mesh_ids
    }

    pub fn scope(&self) -> &SelectionScope {
        &self.scope
    }
}
