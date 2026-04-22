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
    root_ids: Vec<NodeId>,
    nodes: Vec<Node>,
}

impl NodeGraph {
    /*
     TODO: Add runtime graph validation.
     Validation should verify that all root, child, and parent NodeIds are in
     bounds, that parent/child relationships are consistent, and that we have a
     defined policy for unreachable nodes before wiring NodeGraph into glTF.
    */
    pub fn new(nodes: Vec<Node>, node_ids: Vec<NodeId>) -> Self {
        Self {
            nodes,
            root_ids: node_ids,
        }
    }

    pub fn flatten(&self) -> Vec<MeshNode> {
        let mut result = Vec::new();

        let _ = self
            .root_ids
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
#[path = "node_tests.rs"]
mod tests;
