use hyakou_core::{selection::structure::SelectionScope, types::ids::MeshId};

use super::{SelectionManager, SelectionTarget};

fn mesh_id(id: &str) -> MeshId {
    MeshId(id.to_string())
}

fn target(id: &str, outline_ids: &[&str]) -> SelectionTarget {
    SelectionTarget::new(
        mesh_id(id),
        outline_ids.iter().map(|id| mesh_id(id)).collect(),
        SelectionScope::Object,
    )
}

#[test]
fn selection_manager_select_and_deselect() {
    let mut manager = SelectionManager::new();
    let selected = target("mesh", &["outline"]);

    manager.select(selected);

    assert_eq!(manager.selected.len(), 1);
    assert_eq!(manager.outline_selection(), &[mesh_id("outline")]);

    manager.deselect(&target("mesh", &["outline"]));

    assert!(manager.selected.is_empty());
    assert!(manager.outline_selection().is_empty());
}

#[test]
fn selection_manager_add_to_collection() {
    let mut manager = SelectionManager::new();

    manager.add_to_collection(target("mesh_a", &["outline_a"]));
    manager.add_to_collection(target("mesh_b", &["outline_b"]));

    assert_eq!(manager.selected.len(), 2);
    assert_eq!(
        manager.outline_selection(),
        &[mesh_id("outline_a"), mesh_id("outline_b")]
    );
}

#[test]
fn selection_manager_clear() {
    let mut manager = SelectionManager::new();
    manager.select(target("mesh", &["outline"]));

    manager.clear();

    assert!(manager.selected.is_empty());
    assert!(manager.outline_selection().is_empty());
}
