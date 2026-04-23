use std::path::PathBuf;

use glam::{Vec2, Vec3, Vec4};

use super::*;

const EPSILON: f32 = 1e-6;

fn loader() -> GLTFLoader {
    GLTFLoader::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

fn load_from_bytes(bytes: Vec<u8>) -> Result<NodeGraph> {
    pollster::block_on(loader().load_from_bytes(bytes))
}

fn assert_loader_error_contains(result: Result<NodeGraph>, expected: &str) {
    match result {
        Ok(_) => panic!("Expected glTF loading to fail"),
        Err(error) => assert!(
            error.to_string().contains(expected),
            "Expected error to contain `{expected}`, got `{error}`"
        ),
    }
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
    let result = loader().ensure_indices_in_range(&[0, 1, 99], 3, 0);

    match result {
        Ok(()) => panic!("Expected loader to reject out-of-range index"),
        Err(error) => assert!(error.to_string().contains(
            "Index out of range for node 0: index buffer entry 2 references vertex 99, but vertex count is 3"
        )),
    }
}

#[test]
fn test_load_from_bytes_builds_scene_hierarchy_transforms() {
    let node_graph = load_from_bytes(
        include_bytes!("../../assets/gltf/test_fixtures/scene_hierarchy.gltf").to_vec(),
    )
    .unwrap();

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
        load_from_bytes(b"this is not a gltf asset".to_vec()),
        "The given slice does not contain any mesh or gltf object!",
    );
}

#[test]
fn test_load_from_bytes_rejects_missing_normals() {
    assert_loader_error_contains(
        load_from_bytes(
            include_bytes!("../../assets/gltf/test_fixtures/missing_normal.gltf").to_vec(),
        ),
        "No normals found for node: 0",
    );
}

#[test]
fn test_load_from_bytes_rejects_missing_positions() {
    assert_loader_error_contains(
        load_from_bytes(
            include_bytes!("../../assets/gltf/test_fixtures/missing_position.gltf").to_vec(),
        ),
        r#"attributes["POSITION"]: Missing data"#,
    );
}

#[test]
fn test_load_from_bytes_rejects_unsupported_primitive_mode() {
    assert_loader_error_contains(
        load_from_bytes(
            include_bytes!("../../assets/gltf/test_fixtures/unsupported_lines_mode.gltf").to_vec(),
        ),
        "Unsupported primitive mode: Lines",
    );
}

#[test]
fn test_load_from_bytes_reports_missing_external_sidecar() {
    assert_loader_error_contains(
        load_from_bytes(
            include_bytes!("../../assets/gltf/test_fixtures/missing_sidecar.gltf").to_vec(),
        ),
        "missing-sidecar.bin",
    );
}

#[test]
fn test_load_from_bytes_generates_indices_for_non_indexed_mesh() {
    let node_graph = load_from_bytes(
        include_bytes!("../../assets/gltf/test_fixtures/non_indexed_mesh.gltf").to_vec(),
    )
    .unwrap();

    let mesh_nodes = node_graph.flatten();

    assert_eq!(mesh_nodes.len(), 1);
    assert_eq!(mesh_nodes[0].vertices.len(), 36);
    assert_eq!(mesh_nodes[0].indices.len(), mesh_nodes[0].vertices.len());
    assert_eq!(mesh_nodes[0].indices[0], 0);
    assert_eq!(mesh_nodes[0].indices[35], 35);
}

#[test]
fn test_load_from_bytes_reads_vertex_colors_defaults_tex_coords_and_base_color() {
    let node_graph = load_from_bytes(
        include_bytes!("../../assets/gltf/test_fixtures/vertex_colors.gltf").to_vec(),
    )
    .unwrap();

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
