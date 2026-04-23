use anyhow::{Context, Result, anyhow};
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
pub(crate) struct PrimitiveContext {
    asset_label: String,
    node_index: usize,
    node_name: Option<String>,
    mesh_index: usize,
    mesh_name: Option<String>,
    primitive_index: usize,
}

pub(super) fn build_node_graph(
    gltf: &gltf::Gltf,
    buffer_data: &[Vec<u8>],
    asset_label: &str,
) -> Result<NodeGraph> {
    let root_nodes = collect_root_nodes(gltf);
    let mut nodes = Vec::new();
    let mut root_ids = Vec::new();

    for root_node in root_nodes {
        root_ids.push(build_node_recursive(
            root_node,
            None,
            &mut nodes,
            buffer_data,
            asset_label,
        )?);
    }

    if nodes.iter().all(|node| node.meshes.is_empty()) {
        return Err(anyhow!(
            "glTF asset `{asset_label}` contains no renderable meshes"
        ));
    }

    Ok(NodeGraph::new(nodes, root_ids))
}

fn collect_root_nodes<'a>(gltf: &'a gltf::Gltf) -> Vec<gltf::Node<'a>> {
    if let Some(default_scene) = gltf.default_scene() {
        default_scene.nodes().collect()
    } else {
        gltf.scenes().flat_map(|scene| scene.nodes()).collect()
    }
}

fn build_node_recursive(
    gltf_node: gltf::Node<'_>,
    parent_id: Option<NodeId>,
    nodes: &mut Vec<Node>,
    buffer_data: &[Vec<u8>],
    asset_label: &str,
) -> Result<NodeId> {
    let local_transform = build_local_transform(&gltf_node);
    let meshes = build_meshes_for_node(&gltf_node, buffer_data, asset_label)?;
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
            build_node_recursive(child_node, Some(node_id), nodes, buffer_data, asset_label)
        })
        .collect::<Result<Vec<_>>>()?;

    nodes[node_id.0].children_ids = child_ids;

    Ok(node_id)
}

fn build_local_transform(gltf_node: &gltf::Node<'_>) -> Transform {
    let (translation, rotation, scale) = gltf_node.transform().decomposed();

    Transform::new(
        Vec3::new(translation[0], translation[1], translation[2]),
        glam::Quat::from_array(rotation).normalize(),
        Vec3::new(scale[0], scale[1], scale[2]),
    )
}

fn build_meshes_for_node(
    gltf_node: &gltf::Node<'_>,
    buffer_data: &[Vec<u8>],
    asset_label: &str,
) -> Result<Vec<Mesh>> {
    let Some(mesh) = gltf_node.mesh() else {
        return Ok(vec![]);
    };

    let mut meshes = Vec::new();
    for primitive in mesh.primitives() {
        let primitive_context = PrimitiveContext {
            asset_label: asset_label.to_owned(),
            node_index: gltf_node.index(),
            node_name: gltf_node.name().map(str::to_owned),
            mesh_index: mesh.index(),
            mesh_name: mesh.name().map(str::to_owned),
            primitive_index: primitive.index(),
        };
        meshes.extend(build_meshes_for_primitive(
            primitive,
            &primitive_context,
            buffer_data,
        )?);
    }

    Ok(meshes)
}

fn build_meshes_for_primitive(
    primitive: gltf::Primitive<'_>,
    primitive_context: &PrimitiveContext,
    buffer_data: &[Vec<u8>],
) -> Result<Vec<Mesh>> {
    match primitive.mode() {
        Mode::Triangles => build_triangle_meshes(primitive, primitive_context, buffer_data),
        mode => Err(anyhow!(
            "Unsupported primitive mode {mode:?} in {}",
            primitive_context.describe()
        )),
    }
}

fn build_triangle_meshes(
    primitive: gltf::Primitive<'_>,
    primitive_context: &PrimitiveContext,
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
            return Err(anyhow!(
                "Missing POSITION attribute in {}",
                primitive_context.describe()
            ));
        }
    };

    let vertex_count = positions.len();

    let indices = match reader.read_indices() {
        Some(idx) => idx.into_u32().collect::<Vec<_>>(),
        None => (0..vertex_count)
            .map(u32::try_from)
            .collect::<Result<Vec<_>, _>>()
            .with_context(|| {
                format!(
                    "Failed to generate indices for non-indexed mesh in {}",
                    primitive_context.describe()
                )
            })?,
    };
    ensure_indices_in_range(&indices, vertex_count, primitive_context)?;

    let normals = match reader.read_normals() {
        Some(normal) => normal
            .map(|iter| Vec3::new(iter[0], iter[1], iter[2]))
            .collect::<Vec<_>>(),
        None => {
            return Err(anyhow!(
                "Missing NORMAL attribute in {}",
                primitive_context.describe()
            ));
        }
    };
    ensure_attribute_count("NORMAL", normals.len(), vertex_count, primitive_context)?;

    let tex_coords = match reader.read_tex_coords(0) {
        Some(tex_coord) => tex_coord
            .into_f32()
            .map(|tx_coords| Vec2::new(tx_coords[0], tx_coords[1]))
            .collect::<Vec<_>>(),
        None => vec![Vec2::ZERO; vertex_count],
    };
    ensure_attribute_count(
        "TEXCOORD_0",
        tex_coords.len(),
        vertex_count,
        primitive_context,
    )?;

    let colors = match reader.read_colors(0) {
        Some(read_colors) => read_colors
            .into_rgba_f32()
            .map(|v| Vec4::new(v[0], v[1], v[2], v[3]))
            .collect::<Vec<_>>(),
        None => vec![Vec4::ONE; vertex_count],
    };
    ensure_attribute_count("COLOR_0", colors.len(), vertex_count, primitive_context)?;

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
        name: primitive_context.mesh_name.clone(),
        vertices,
        indices,
    }])
}

fn ensure_attribute_count(
    attribute_name: &str,
    actual_count: usize,
    vertex_count: usize,
    primitive_context: &PrimitiveContext,
) -> Result<()> {
    if actual_count != vertex_count {
        return Err(anyhow!(
            "Attribute `{attribute_name}` count mismatch in {}: expected {vertex_count}, got {actual_count}",
            primitive_context.describe()
        ));
    }

    Ok(())
}

pub(crate) fn ensure_indices_in_range(
    indices: &[u32],
    vertex_count: usize,
    primitive_context: &PrimitiveContext,
) -> Result<()> {
    let vertex_count_u32 = u32::try_from(vertex_count).map_err(|_| {
        anyhow!(
            "Vertex count exceeds supported indexed range in {}: {vertex_count}",
            primitive_context.describe()
        )
    })?;

    for (position, index) in indices.iter().copied().enumerate() {
        if index >= vertex_count_u32 {
            return Err(anyhow!(
                "Index out of range in {}: index buffer entry {position} references vertex {index}, but vertex count is {vertex_count}",
                primitive_context.describe()
            ));
        }
    }

    Ok(())
}

impl PrimitiveContext {
    #[cfg(test)]
    pub(crate) fn new_for_test(
        asset_label: String,
        node_index: usize,
        node_name: Option<String>,
        mesh_index: usize,
        mesh_name: Option<String>,
        primitive_index: usize,
    ) -> Self {
        Self {
            asset_label,
            node_index,
            node_name,
            mesh_index,
            mesh_name,
            primitive_index,
        }
    }

    fn describe(&self) -> String {
        format!(
            "asset `{}`, node {}{}, mesh {}{}, primitive {}",
            self.asset_label,
            self.node_index,
            Self::optional_name(self.node_name.as_deref()),
            self.mesh_index,
            Self::optional_name(self.mesh_name.as_deref()),
            self.primitive_index
        )
    }

    fn optional_name(name: Option<&str>) -> String {
        name.map(|name| format!(" `{name}`")).unwrap_or_default()
    }
}
