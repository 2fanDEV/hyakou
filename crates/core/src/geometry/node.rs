use std::ops::Deref;

use glam::Mat4;

use crate::{components::mesh_node::MeshNode, geometry::mesh::Mesh, types::transform::Transform};

#[derive(Debug, Clone, Copy)]
pub struct NodeId(pub usize);
impl Deref for NodeId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct NodeGraph {
    node_ids: Vec<NodeId>,
    nodes: Vec<Node>,
}

impl NodeGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            node_ids: Vec::new(),
        }
    }

    pub fn flatten(&self) -> Vec<MeshNode> {
        let mut result = Vec::new();

        let _ = self
            .node_ids
            .iter()
            .map(|nd| self.flatten_nodes(*nd, &Mat4::IDENTITY, &mut result))
            .collect::<Vec<_>>();
        result
    }

    pub fn flatten_nodes(&self, node_id: NodeId, parent_world: &Mat4, out: &mut Vec<MeshNode>) {
        let node = &self.nodes[*node_id];
        let local = node.local_transform.get_matrix();
        let world = parent_world * local;

        let (scale, rotation, translation) = world.to_scale_rotation_translation();
        let world_transform = Transform::new(translation, rotation, scale);

        for mesh in &node.meshes {
            out.push(MeshNode::new(mesh.clone(), world_transform));
        }

        for child_node_id in &node.children_ids {
            self.flatten_nodes(*child_node_id, &world, out);
        }
    }
}

pub struct Node {
    pub local_transform: Transform,
    pub meshes: Vec<Mesh>,
    pub children_ids: Vec<NodeId>,
    pub parent_id: Option<NodeId>,
}

#[cfg(test)]
mod tests {
    use glam::{Quat, Vec3};

    use super::{Node, NodeGraph, NodeId};
    use crate::{geometry::mesh::Mesh, types::transform::Transform};

    const EPSILON: f32 = 1e-6;

    fn test_mesh(name: &str) -> Mesh {
        Mesh::new(Some(name.to_string()), vec![], vec![])
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
        let graph = NodeGraph::new();

        let result = graph.flatten();

        assert!(result.is_empty());
    }

    #[test]
    fn flatten_single_root_with_one_mesh_returns_one_mesh_node() {
        let graph = NodeGraph {
            node_ids: vec![NodeId(0)],
            nodes: vec![Node {
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
            node_ids: vec![NodeId(0)],
            nodes: vec![Node {
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
            node_ids: vec![NodeId(0)],
            nodes: vec![
                Node {
                    local_transform: test_transform(10.0, 0.0, 0.0),
                    meshes: vec![test_mesh("parent")],
                    children_ids: vec![NodeId(1)],
                    parent_id: None,
                },
                Node {
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
}
