use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use glam::{Vec2, Vec3, Vec4};
use gltf::mesh::Mode;
use hyakou_core::{
    geometry::{
        mesh::Mesh,
        node::{Node, NodeGraph, NodeId},
        vertices::Vertex,
    },
    types::transform::Transform,
};

#[derive(Debug, Clone)]
pub struct GLTFLoader {
    BASE_PATH: PathBuf,
}

impl GLTFLoader {
    pub fn new(path: PathBuf) -> Self {
        Self { BASE_PATH: path }
    }

    #[cfg(target_arch = "wasm32")]
    async fn read_bytes(&self, path: &Path) -> Result<Vec<u8>> {
        use gloo_net::http::Request;
        let slice = Request::get(path.to_str().unwrap()).build()?;
        let response = slice.send().await?;
        let slice = match response.binary().await {
            Ok(s) => s,
            Err(e) => return Err(anyhow!(e)),
        };
        Ok(slice)
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn read_bytes(&self, path: &Path) -> Result<Vec<u8>> {
        std::fs::read(path)
            .map_err(|error| anyhow!("Failed to read glTF buffer `{}`: {error}", path.display()))
    }

    pub async fn load_from_path(&self, path: &Path) -> Result<NodeGraph> {
        let slice = match self.read_bytes(path).await {
            Ok(slice) => slice,
            Err(e) => return Err(e),
        };
        self.load_from_bytes(slice).await
    }

    pub async fn load_from_bytes(&self, slice: Vec<u8>) -> Result<NodeGraph> {
        let gltf = match gltf::Gltf::from_slice(&slice) {
            Ok(gltf) => gltf,
            Err(error) => {
                return Err(anyhow!(
                    "The given slice does not contain any mesh or gltf object! {error}"
                ));
            }
        };

        let buffer_data = self.load_buffer_data(&gltf).await?;
        let root_nodes = self.collect_root_nodes(&gltf);

        let mut nodes = Vec::new();
        let mut root_ids = Vec::new();

        for root_node in root_nodes {
            root_ids.push(self.build_node_recursive(root_node, None, &mut nodes, &buffer_data)?);
        }

        Ok(NodeGraph::new(nodes, root_ids))
    }

    async fn load_buffer_data(&self, gltf: &gltf::Gltf) -> Result<Vec<Vec<u8>>> {
        let buffer_async_handles = gltf.buffers().map(async |buffer| match buffer.source() {
            gltf::buffer::Source::Bin => gltf
                .blob
                .clone()
                .ok_or_else(|| anyhow!("Glb blob is missing")),
            gltf::buffer::Source::Uri(uri) => {
                let uri = self.BASE_PATH.join("assets/gltf").join(uri);
                self.read_bytes(&uri).await
            }
        });

        futures::future::try_join_all(buffer_async_handles).await
    }

    fn collect_root_nodes<'a>(&self, gltf: &'a gltf::Gltf) -> Vec<gltf::Node<'a>> {
        if let Some(default_scene) = gltf.default_scene() {
            default_scene.nodes().collect()
        } else {
            gltf.scenes().flat_map(|scene| scene.nodes()).collect()
        }
    }

    fn build_node_recursive(
        &self,
        gltf_node: gltf::Node<'_>,
        parent_id: Option<NodeId>,
        nodes: &mut Vec<Node>,
        buffer_data: &[Vec<u8>],
    ) -> Result<NodeId> {
        let local_transform = self.build_local_transform(&gltf_node);
        let meshes = self.build_meshes_for_node(&gltf_node, buffer_data)?;
        let node_id = NodeId(nodes.len());

        nodes.push(Node {
            local_transform,
            meshes,
            children_ids: vec![],
            parent_id,
        });

        let child_ids = gltf_node
            .children()
            .map(|child_node| {
                self.build_node_recursive(child_node, Some(node_id), nodes, buffer_data)
            })
            .collect::<Result<Vec<_>>>()?;

        nodes[node_id.0].children_ids = child_ids;

        Ok(node_id)
    }

    fn build_local_transform(&self, gltf_node: &gltf::Node<'_>) -> Transform {
        let (translation, rotation, scale) = gltf_node.transform().decomposed();

        Transform::new(
            Vec3::new(translation[0], translation[1], translation[2]),
            glam::Quat::from_array(rotation).normalize(),
            Vec3::new(scale[0], scale[1], scale[2]),
        )
    }

    fn build_meshes_for_node(
        &self,
        gltf_node: &gltf::Node<'_>,
        buffer_data: &[Vec<u8>],
    ) -> Result<Vec<Mesh>> {
        let Some(mesh) = gltf_node.mesh() else {
            return Ok(vec![]);
        };

        let mut meshes = Vec::new();
        for primitive in mesh.primitives() {
            meshes.extend(self.build_meshes_for_primitive(
                primitive,
                mesh.name(),
                gltf_node.index(),
                buffer_data,
            )?);
        }

        Ok(meshes)
    }

    fn build_meshes_for_primitive(
        &self,
        primitive: gltf::Primitive<'_>,
        mesh_name: Option<&str>,
        node_index: usize,
        buffer_data: &[Vec<u8>],
    ) -> Result<Vec<Mesh>> {
        match primitive.mode() {
            Mode::Triangles => {
                self.build_triangle_meshes(primitive, mesh_name, node_index, buffer_data)
            }
            mode => Err(anyhow!("Unsupported primitive mode: {mode:?}")),
        }
    }

    fn build_triangle_meshes(
        &self,
        primitive: gltf::Primitive<'_>,
        mesh_name: Option<&str>,
        node_index: usize,
        buffer_data: &[Vec<u8>],
    ) -> Result<Vec<Mesh>> {
        let base_color_factor = primitive
            .material()
            .pbr_metallic_roughness()
            .base_color_factor();

        let base_color_factor_vec4 = Vec4::new(
            base_color_factor[0],
            base_color_factor[1],
            base_color_factor[2],
            base_color_factor[3],
        );

        let reader = primitive.reader(|buffer| {
            let index = buffer.index();
            buffer_data.get(index).map(|data| data.as_slice())
        });

        let positions = match reader.read_positions() {
            Some(pos) => pos
                .map(|iter| Vec3::new(iter[0], iter[1], iter[2]))
                .collect::<Vec<_>>(),
            None => {
                return Err(anyhow!("No positions found for node: {node_index}"));
            }
        };

        let vertex_count = positions.len();

        let indices = match reader.read_indices() {
            Some(idx) => idx.into_u32().collect::<Vec<_>>(),
            None => (0..vertex_count)
                .map(u32::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        };
        self.ensure_indices_in_range(&indices, vertex_count, node_index)?;

        let normals = match reader.read_normals() {
            Some(normal) => normal
                .map(|iter| Vec3::new(iter[0], iter[1], iter[2]))
                .collect::<Vec<_>>(),
            None => {
                return Err(anyhow!("No normals found for node: {node_index}"));
            }
        };
        self.ensure_attribute_count("normals", normals.len(), vertex_count, node_index)?;

        let tex_coords = match reader.read_tex_coords(0) {
            Some(tex_coord) => tex_coord
                .into_f32()
                .map(|tx_coords| Vec2::new(tx_coords[0], tx_coords[1]))
                .collect::<Vec<_>>(),
            None => vec![Vec2::ZERO; vertex_count],
        };
        self.ensure_attribute_count("tex_coords", tex_coords.len(), vertex_count, node_index)?;

        let colors = match reader.read_colors(0) {
            Some(read_colors) => read_colors
                .into_rgba_f32()
                .map(|v| Vec4::new(v[0], v[1], v[2], v[3]))
                .collect::<Vec<_>>(),
            None => vec![Vec4::ONE; vertex_count],
        };
        self.ensure_attribute_count("colors", colors.len(), vertex_count, node_index)?;

        let vertices = (0..vertex_count)
            .map(|i| {
                Vertex::new(
                    positions[i],
                    tex_coords[i],
                    normals[i],
                    colors[i],
                    base_color_factor_vec4,
                )
            })
            .collect::<Vec<_>>();

        Ok(vec![Mesh {
            name: mesh_name.map(|name| name.to_owned()),
            vertices,
            indices,
        }])
    }

    fn ensure_attribute_count(
        &self,
        attribute_name: &str,
        actual_count: usize,
        vertex_count: usize,
        node_index: usize,
    ) -> Result<()> {
        if actual_count != vertex_count {
            return Err(anyhow!(
                "Attribute `{attribute_name}` count mismatch for node {node_index}: expected {vertex_count}, got {actual_count}"
            ));
        }

        Ok(())
    }

    fn ensure_indices_in_range(
        &self,
        indices: &[u32],
        vertex_count: usize,
        node_index: usize,
    ) -> Result<()> {
        let vertex_count_u32 = u32::try_from(vertex_count).map_err(|_| {
            anyhow!(
                "Vertex count exceeds supported indexed range for node {node_index}: {vertex_count}"
            )
        })?;

        for (position, index) in indices.iter().copied().enumerate() {
            if index >= vertex_count_u32 {
                return Err(anyhow!(
                    "Index out of range for node {node_index}: index buffer entry {position} references vertex {index}, but vertex count is {vertex_count}"
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "gltf_tests.rs"]
mod tests;
