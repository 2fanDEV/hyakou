use hyakou_core::{selection::structure::SelectionTarget, types::ids::MeshId};

pub struct SelectionManager {
    selected: Vec<SelectionTarget>,
    outline_selected: Vec<MeshId>,
}

impl SelectionManager {
    pub fn new() -> Self {
        Self {
            selected: Vec::new(),
            outline_selected: Vec::new(),
        }
    }

    pub fn current_selection(&self) -> &[SelectionTarget] {
        &self.selected
    }

    pub fn current_outline_selection(&self) -> &[MeshId] {
        &self.outline_selected
    }

    pub fn select_single(&mut self, target: SelectionTarget) {
        self.selected.clear();
        self.outline_selected.clear();
        self.outline_selected
            .extend_from_slice(target.outline_mesh_ids());
        self.selected.push(target);
    }

    pub fn add_selection(&mut self, target: SelectionTarget) {
        self.outline_selected
            .extend_from_slice(target.outline_mesh_ids());
        self.selected.push(target);
    }

    pub fn deselect(&mut self, target: SelectionTarget) {
        if let Some(index) = self
            .selected
            .iter()
            .position(|s| s.mesh_id() == target.mesh_id())
        {
            self.selected.remove(index);
            self.outline_selected
                .retain(|mesh_id| !target.outline_mesh_ids().contains(mesh_id));
        }
    }

    pub fn clear(&mut self) {
        self.selected.clear();
        self.outline_selected.clear();
    }

    pub fn current_selected(&self) -> &[SelectionTarget] {
        &self.selected
    }

    pub fn current_outline_selected(&self) -> &[MeshId] {
        &self.outline_selected
    }
}
