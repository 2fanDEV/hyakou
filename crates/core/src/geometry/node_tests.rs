use glam::{Quat, Vec3};

use super::{Node, NodeGraph, NodeId, NodeMetadata};
use crate::{geometry::mesh::Mesh, types::transform::Transform};

const EPSILON: f32 = 1e-6;

fn test_mesh(name: &str) -> Mesh {
    Mesh::new(Some(name.to_string()), None, vec![], vec![])
}

fn test_transform(x: f32, y: f32, z: f32) -> Transform {
    Transform::new(Vec3::new(x, y, z), Quat::IDENTITY, Vec3::ONE)
}

fn assert_vec3_eq(actual: Vec3, expected: Vec3, message: &str) {
    assert!(
        (actual - expected).length() < EPSILON,
        "{}: expected {:?}, got {:?}",
        message,
        expected,
        actual
    );
}

fn assert_quat_eq(actual: Quat, expected: Quat, message: &str) {
    let dot = actual.dot(expected).abs();
    assert!(
        (dot - 1.0).abs() < EPSILON,
        "{}: expected {:?}, got {:?} (dot: {})",
        message,
        expected,
        actual,
        dot
    );
}

#[test]
fn flatten_empty_graph_returns_empty_vec() {
    let graph = NodeGraph::new(vec![], vec![]);
    let result = graph.flatten();
    assert!(result.is_empty());
}

#[test]
fn flatten_single_root_with_one_mesh_returns_one_mesh_node() {
    let graph = NodeGraph {
        root_ids: vec![NodeId(0)],
        nodes: vec![Node {
            metadata: NodeMetadata::default(),
            local_transform: test_transform(1.0, 2.0, 3.0),
            meshes: vec![test_mesh("root")],
            children_ids: vec![],
            parent_id: None,
        }],
    };

    let result = graph.flatten();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name.as_deref(), Some("root"));
    assert_vec3_eq(
        result[0].transform.position,
        Vec3::new(1.0, 2.0, 3.0),
        "single root world position",
    );
    assert_quat_eq(
        result[0].transform.rotation,
        Quat::IDENTITY,
        "single root world rotation",
    );
    assert_vec3_eq(
        result[0].transform.scale,
        Vec3::ONE,
        "single root world scale",
    );
}

#[test]
fn flatten_single_node_with_multiple_meshes_returns_multiple_mesh_nodes() {
    let graph = NodeGraph {
        root_ids: vec![NodeId(0)],
        nodes: vec![Node {
            metadata: NodeMetadata::default(),
            local_transform: test_transform(4.0, 5.0, 6.0),
            meshes: vec![test_mesh("mesh_a"), test_mesh("mesh_b")],
            children_ids: vec![],
            parent_id: None,
        }],
    };

    let result = graph.flatten();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name.as_deref(), Some("mesh_a"));
    assert_eq!(result[1].name.as_deref(), Some("mesh_b"));
    assert_vec3_eq(
        result[0].transform.position,
        Vec3::new(4.0, 5.0, 6.0),
        "first mesh world position",
    );
    assert_vec3_eq(
        result[1].transform.position,
        Vec3::new(4.0, 5.0, 6.0),
        "second mesh world position",
    );
}

#[test]
fn flatten_accumulates_parent_and_child_transforms() {
    let graph = NodeGraph {
        root_ids: vec![NodeId(0)],
        nodes: vec![
            Node {
                metadata: NodeMetadata::default(),
                local_transform: test_transform(10.0, 0.0, 0.0),
                meshes: vec![test_mesh("parent")],
                children_ids: vec![NodeId(1)],
                parent_id: None,
            },
            Node {
                metadata: NodeMetadata::default(),
                local_transform: test_transform(2.0, 0.0, 0.0),
                meshes: vec![test_mesh("child")],
                children_ids: vec![],
                parent_id: Some(NodeId(0)),
            },
        ],
    };

    let result = graph.flatten();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name.as_deref(), Some("parent"));
    assert_eq!(result[1].name.as_deref(), Some("child"));
    assert_vec3_eq(
        result[0].transform.position,
        Vec3::new(10.0, 0.0, 0.0),
        "parent world position",
    );
    assert_vec3_eq(
        result[1].transform.position,
        Vec3::new(12.0, 0.0, 0.0),
        "child world position should include parent translation",
    );
}

#[test]
fn flatten_two_independent_roots_returns_mesh_nodes_for_both_roots() {
    let graph = NodeGraph {
        root_ids: vec![NodeId(0), NodeId(1)],
        nodes: vec![
            Node {
                metadata: NodeMetadata::default(),
                local_transform: test_transform(1.0, 0.0, 0.0),
                meshes: vec![test_mesh("root_a")],
                children_ids: vec![],
                parent_id: None,
            },
            Node {
                metadata: NodeMetadata::default(),
                local_transform: test_transform(5.0, 0.0, 0.0),
                meshes: vec![test_mesh("root_b")],
                children_ids: vec![],
                parent_id: None,
            },
        ],
    };

    let result = graph.flatten();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name.as_deref(), Some("root_a"));
    assert_eq!(result[1].name.as_deref(), Some("root_b"));
    assert_vec3_eq(
        result[0].transform.position,
        Vec3::new(1.0, 0.0, 0.0),
        "first root world position",
    );
    assert_vec3_eq(
        result[1].transform.position,
        Vec3::new(5.0, 0.0, 0.0),
        "second root world position",
    );
}

#[test]
fn flatten_accumulates_parent_child_and_grandchild_transforms() {
    let graph = NodeGraph {
        root_ids: vec![NodeId(0)],
        nodes: vec![
            Node {
                metadata: NodeMetadata::default(),
                local_transform: test_transform(10.0, 0.0, 0.0),
                meshes: vec![test_mesh("parent")],
                children_ids: vec![NodeId(1)],
                parent_id: None,
            },
            Node {
                metadata: NodeMetadata::default(),
                local_transform: test_transform(2.0, 0.0, 0.0),
                meshes: vec![test_mesh("child")],
                children_ids: vec![NodeId(2)],
                parent_id: Some(NodeId(0)),
            },
            Node {
                metadata: NodeMetadata::default(),
                local_transform: test_transform(3.0, 0.0, 0.0),
                meshes: vec![test_mesh("grandchild")],
                children_ids: vec![],
                parent_id: Some(NodeId(1)),
            },
        ],
    };

    let result = graph.flatten();

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].name.as_deref(), Some("parent"));
    assert_eq!(result[1].name.as_deref(), Some("child"));
    assert_eq!(result[2].name.as_deref(), Some("grandchild"));
    assert_vec3_eq(
        result[0].transform.position,
        Vec3::new(10.0, 0.0, 0.0),
        "parent world position",
    );
    assert_vec3_eq(
        result[1].transform.position,
        Vec3::new(12.0, 0.0, 0.0),
        "child world position",
    );
    assert_vec3_eq(
        result[2].transform.position,
        Vec3::new(15.0, 0.0, 0.0),
        "grandchild world position should include parent and child translation",
    );
}
