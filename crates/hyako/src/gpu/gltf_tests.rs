use std::path::PathBuf;

use glam::{Vec2, Vec3, Vec4};

use super::*;

const EPSILON: f32 = 1e-6;

fn loader() -> GLTFLoader {
    GLTFLoader::new()
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/gltf/test_fixtures")
        .join(name)
}

fn load_from_bytes(bytes: Vec<u8>) -> Result<NodeGraph> {
    pollster::block_on(loader().load_from_bytes(bytes))
}

fn load_from_path(name: &str) -> Result<NodeGraph> {
    pollster::block_on(loader().load_from_path(&fixture_path(name)))
}

fn assert_loader_error_contains(result: Result<NodeGraph>, expected: &str) {
    match result {
        Ok(_) => panic!("Expected glTF loading to fail"),
        Err(error) => assert!(
            format!("{error:?}").contains(expected),
            "Expected error to contain `{expected}`, got `{error}`"
        ),
    }
}

fn primitive_context() -> PrimitiveContext {
    PrimitiveContext::new_for_test(
        "test asset".to_string(),
        0,
        Some("TestNode".to_string()),
        0,
        Some("TestMesh".to_string()),
        0,
    )
}

fn assert_vec2_eq(actual: Vec2, expected: Vec2, message: &str) {
    assert!(
        (actual - expected).length() < EPSILON,
        "{message}: expected {expected:?}, got {actual:?}"
    );
}

fn assert_vec3_eq(actual: Vec3, expected: Vec3, message: &str) {
    assert!(
        (actual - expected).length() < EPSILON,
        "{message}: expected {expected:?}, got {actual:?}"
    );
}

fn assert_vec4_eq(actual: Vec4, expected: Vec4, message: &str) {
    assert!(
        (actual - expected).length() < EPSILON,
        "{message}: expected {expected:?}, got {actual:?}"
    );
}

#[test]
fn test_ensure_indices_in_range_rejects_out_of_range_index() {
    let context = primitive_context();
    let result = ensure_indices_in_range(&[0, 1, 99], 3, &context);

    match result {
        Ok(()) => panic!("Expected loader to reject out-of-range index"),
        Err(error) => assert!(error.to_string().contains(
            "Index out of range in asset `test asset`, node 0 `TestNode`, mesh 0 `TestMesh`, primitive 0: index buffer entry 2 references vertex 99, but vertex count is 3"
        )),
    }
}

#[test]
fn test_load_from_path_builds_scene_hierarchy_transforms() {
    let node_graph = load_from_path("scene_hierarchy.gltf").unwrap();

    let mesh_nodes = node_graph.flatten();

    assert_eq!(mesh_nodes.len(), 2);
    assert_vec3_eq(
        mesh_nodes[0].transform.position,
        Vec3::new(10.0, 0.0, 0.0),
        "root mesh world position",
    );
    assert_vec3_eq(
        mesh_nodes[1].transform.position,
        Vec3::new(12.0, 0.0, 0.0),
        "child mesh world position",
    );
}

#[test]
fn test_load_from_bytes_rejects_malformed_bytes() {
    assert_loader_error_contains(
        pollster::block_on(
            loader().load_from_bytes_with_label(b"this is not a gltf asset".to_vec(), "bad.gltf"),
        ),
        "Failed to parse glTF asset `bad.gltf`",
    );
}

#[test]
fn test_load_from_path_rejects_missing_normals() {
    assert_loader_error_contains(
        load_from_path("missing_normal.gltf"),
        "Missing NORMAL attribute in asset",
    );
}

#[test]
fn test_load_from_path_rejects_missing_positions_with_context() {
    assert_loader_error_contains(
        load_from_path("missing_position.gltf"),
        "Failed to parse glTF asset",
    );
    assert_loader_error_contains(
        load_from_path("missing_position.gltf"),
        r#"attributes["POSITION"]: Missing data"#,
    );
}

#[test]
fn test_load_from_path_rejects_unsupported_primitive_mode() {
    assert_loader_error_contains(
        load_from_path("unsupported_lines_mode.gltf"),
        "Unsupported primitive mode Lines in asset",
    );
}

#[test]
fn test_load_from_path_reports_missing_external_sidecar() {
    assert_loader_error_contains(
        load_from_path("missing_sidecar.gltf"),
        "missing-sidecar.bin",
    );
}

#[test]
fn test_load_from_bytes_rejects_relative_external_buffer() {
    assert_loader_error_contains(
        load_from_bytes(
            include_bytes!("../../assets/gltf/test_fixtures/vertex_colors.gltf").to_vec(),
        ),
        "cannot be resolved from in-memory glTF bytes",
    );
}

#[test]
fn test_load_from_path_generates_indices_for_non_indexed_mesh() {
    let node_graph = load_from_path("non_indexed_mesh.gltf").unwrap();

    let mesh_nodes = node_graph.flatten();

    assert_eq!(mesh_nodes.len(), 1);
    assert_eq!(mesh_nodes[0].vertices.len(), 36);
    assert_eq!(mesh_nodes[0].indices.len(), mesh_nodes[0].vertices.len());
    assert_eq!(mesh_nodes[0].indices[0], 0);
    assert_eq!(mesh_nodes[0].indices[35], 35);
}

#[test]
fn test_load_from_path_reads_vertex_colors_defaults_tex_coords_and_base_color() {
    let node_graph = load_from_path("vertex_colors.gltf").unwrap();

    let mesh_nodes = node_graph.flatten();
    let vertices = &mesh_nodes[0].vertices;

    assert_eq!(mesh_nodes.len(), 1);
    assert_eq!(vertices.len(), 3);
    assert_vec4_eq(
        vertices[0].colors,
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        "first vertex color",
    );
    assert_vec4_eq(
        vertices[1].colors,
        Vec4::new(0.0, 1.0, 0.0, 0.5),
        "second vertex color",
    );
    assert_vec4_eq(
        vertices[2].colors,
        Vec4::new(0.0, 0.0, 1.0, 0.25),
        "third vertex color",
    );

    for vertex in vertices {
        assert_vec2_eq(vertex.tex_coords, Vec2::ZERO, "default tex coords");
        assert_vec4_eq(
            vertex.pbr_base_color_factor,
            Vec4::new(0.25, 0.5, 0.75, 1.0),
            "base color factor",
        );
    }
}

#[test]
fn test_load_from_bytes_rejects_asset_without_renderable_meshes() {
    let bytes = br#"{
        "asset": { "version": "2.0" },
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": [{ "name": "EmptyNode" }]
    }"#
    .to_vec();

    assert_loader_error_contains(
        pollster::block_on(loader().load_from_bytes_with_label(bytes, "empty.gltf")),
        "glTF asset `empty.gltf` contains no renderable meshes",
    );
}
