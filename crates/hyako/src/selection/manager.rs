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

    pub fn select(&mut self, target: SelectionTarget) {
        self.selected.clear();
        self.outline_selected.clear();
        self.add_to_collection(target);
    }

    pub fn add_to_collection(&mut self, target: SelectionTarget) {
        self.selected.push(target);
        self.rebuild_outline_selection();
    }

    pub fn deselect(&mut self, target: &SelectionTarget) {
        self.selected
            .retain(|selected| selected.mesh_id() != target.mesh_id());
        self.rebuild_outline_selection();
    }

    pub fn outline_selection(&self) -> &[MeshId] {
        &self.outline_selected
    }

    pub fn clear(&mut self) {
        self.selected.clear();
        self.outline_selected.clear();
    }

    fn rebuild_outline_selection(&mut self) {
        self.outline_selected.clear();
        for target in &self.selected {
            self.outline_selected
                .extend_from_slice(target.outline_mesh_ids());
        }
    }
}

#[cfg(test)]
#[path = "manager_tests.rs"]
mod manager_tests;
