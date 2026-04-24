use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

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

fn load_from_bytes(bytes: Vec<u8>) -> Result<ImportedScene> {
    pollster::block_on(loader().load_from_bytes(bytes))
}

fn load_from_path(name: &str) -> Result<ImportedScene> {
    pollster::block_on(loader().load_from_path(&fixture_path(name)))
}

fn load_from_bundle(entry_name: &str, file_names: &[&str]) -> Result<ImportedScene> {
    let files = file_names
        .iter()
        .map(|name| (name.to_string(), std::fs::read(fixture_path(name)).unwrap()))
        .collect();

    pollster::block_on(loader().load_from_file_bundle(entry_name, files))
}

fn load_glb_from_path(bytes: Vec<u8>) -> Result<ImportedScene> {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("hyako_vertex_colors_{suffix}.glb"));
    fs::write(&path, bytes).unwrap();

    let result = pollster::block_on(loader().load_from_path(&path));
    fs::remove_file(path).unwrap();
    result
}

fn vertex_colors_glb_bytes() -> Vec<u8> {
    let json = br#"{
  "asset": { "version": "2.0" },
  "scene": 0,
  "scenes": [{ "nodes": [0] }],
  "nodes": [{ "mesh": 0, "name": "VertexColorsGlb" }],
  "meshes": [{
    "name": "VertexColorsGlb",
    "primitives": [{
      "attributes": { "POSITION": 0, "NORMAL": 1, "COLOR_0": 2 },
      "indices": 3,
      "material": 0,
      "mode": 4
    }]
  }],
  "materials": [{
    "pbrMetallicRoughness": {
      "baseColorFactor": [0.25, 0.5, 0.75, 1.0]
    }
  }],
  "buffers": [{ "byteLength": 128 }],
  "bufferViews": [
    { "buffer": 0, "byteLength": 36, "byteOffset": 8, "target": 34962 },
    { "buffer": 0, "byteLength": 36, "byteOffset": 44, "target": 34962 },
    { "buffer": 0, "byteLength": 48, "byteOffset": 80, "target": 34962 },
    { "buffer": 0, "byteLength": 6, "byteOffset": 0, "target": 34963 }
  ],
  "accessors": [
    { "bufferView": 0, "byteOffset": 0, "componentType": 5126, "count": 3, "max": [1.0, 1.0, 0.0], "min": [0.0, 0.0, 0.0], "type": "VEC3" },
    { "bufferView": 1, "byteOffset": 0, "componentType": 5126, "count": 3, "type": "VEC3" },
    { "bufferView": 2, "byteOffset": 0, "componentType": 5126, "count": 3, "type": "VEC4" },
    { "bufferView": 3, "byteOffset": 0, "componentType": 5123, "count": 3, "max": [2], "min": [0], "type": "SCALAR" }
  ]
}"#;
    let bin = include_bytes!("../../assets/gltf/test_fixtures/vertex_colors.bin");

    let mut json_chunk = json.to_vec();
    while json_chunk.len() % 4 != 0 {
        json_chunk.push(b' ');
    }

    let mut bin_chunk = bin.to_vec();
    while bin_chunk.len() % 4 != 0 {
        bin_chunk.push(0);
    }

    let total_len = 12 + 8 + json_chunk.len() + 8 + bin_chunk.len();
    let mut glb = Vec::with_capacity(total_len);
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2_u32.to_le_bytes());
    glb.extend_from_slice(&(total_len as u32).to_le_bytes());
    glb.extend_from_slice(&(json_chunk.len() as u32).to_le_bytes());
    glb.extend_from_slice(b"JSON");
    glb.extend_from_slice(&json_chunk);
    glb.extend_from_slice(&(bin_chunk.len() as u32).to_le_bytes());
    glb.extend_from_slice(b"BIN\0");
    glb.extend_from_slice(&bin_chunk);
    glb
}

fn assert_loader_error_contains(result: Result<ImportedScene>, expected: &str) {
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
    let imported_scene = load_from_path("scene_hierarchy.gltf").unwrap();

    let mesh_nodes = imported_scene.node_graph.flatten();

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
    let imported_scene = load_from_path("non_indexed_mesh.gltf").unwrap();

    let mesh_nodes = imported_scene.node_graph.flatten();

    assert_eq!(mesh_nodes.len(), 1);
    assert_eq!(mesh_nodes[0].vertices.len(), 36);
    assert_eq!(mesh_nodes[0].indices.len(), mesh_nodes[0].vertices.len());
    assert_eq!(mesh_nodes[0].indices[0], 0);
    assert_eq!(mesh_nodes[0].indices[35], 35);
}

#[test]
fn test_load_from_path_reads_vertex_colors_defaults_tex_coords_and_base_color() {
    let imported_scene = load_from_path("vertex_colors.gltf").unwrap();

    let mesh_nodes = imported_scene.node_graph.flatten();
    let vertices = &mesh_nodes[0].vertices;

    assert_eq!(mesh_nodes.len(), 1);
    assert_eq!(vertices.len(), 3);
    assert_eq!(mesh_nodes[0].material_index, Some(0));
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
    }

    assert_eq!(imported_scene.materials.len(), 1);
    assert_vec4_eq(
        imported_scene.materials[0].base_color_factor,
        Vec4::new(0.25, 0.5, 0.75, 1.0),
        "base color factor",
    );
}

#[test]
fn test_load_from_path_reads_data_uri_buffer() {
    let imported_scene = load_from_path("vertex_colors_data_uri.gltf").unwrap();

    let mesh_nodes = imported_scene.node_graph.flatten();
    let vertices = &mesh_nodes[0].vertices;

    assert_eq!(mesh_nodes.len(), 1);
    assert_eq!(vertices.len(), 3);
    assert_eq!(mesh_nodes[0].material_index, Some(0));
    assert_vec4_eq(
        vertices[0].colors,
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        "first data URI vertex color",
    );
    assert_vec4_eq(
        vertices[1].colors,
        Vec4::new(0.0, 1.0, 0.0, 0.5),
        "second data URI vertex color",
    );
    assert_vec4_eq(
        vertices[2].colors,
        Vec4::new(0.0, 0.0, 1.0, 0.25),
        "third data URI vertex color",
    );
}

#[test]
fn test_load_from_bytes_reads_data_uri_buffer() {
    let imported_scene = load_from_bytes(
        include_bytes!("../../assets/gltf/test_fixtures/vertex_colors_data_uri.gltf").to_vec(),
    )
    .unwrap();

    let mesh_nodes = imported_scene.node_graph.flatten();
    let vertices = &mesh_nodes[0].vertices;

    assert_eq!(mesh_nodes.len(), 1);
    assert_eq!(vertices.len(), 3);
    assert_vec4_eq(
        vertices[0].colors,
        Vec4::new(1.0, 0.0, 0.0, 1.0),
        "first byte-loaded data URI vertex color",
    );
}

#[test]
fn test_load_from_path_reads_glb_embedded_buffer() {
    let imported_scene = load_glb_from_path(vertex_colors_glb_bytes()).unwrap();

    let mesh_nodes = imported_scene.node_graph.flatten();
    let vertices = &mesh_nodes[0].vertices;

    assert_eq!(mesh_nodes.len(), 1);
    assert_eq!(vertices.len(), 3);
    assert_vec4_eq(
        vertices[1].colors,
        Vec4::new(0.0, 1.0, 0.0, 0.5),
        "second path-loaded glb vertex color",
    );
}

#[test]
fn test_load_from_bytes_reads_glb_embedded_buffer() {
    let imported_scene = load_from_bytes(vertex_colors_glb_bytes()).unwrap();

    let mesh_nodes = imported_scene.node_graph.flatten();
    let vertices = &mesh_nodes[0].vertices;

    assert_eq!(mesh_nodes.len(), 1);
    assert_eq!(vertices.len(), 3);
    assert_vec4_eq(
        vertices[2].colors,
        Vec4::new(0.0, 0.0, 1.0, 0.25),
        "third byte-loaded glb vertex color",
    );
}

#[test]
fn test_load_from_path_reads_inline_material_texture_image_and_sampler() {
    let imported_scene = load_from_path("material_texture_data_uri.gltf").unwrap();

    assert_eq!(imported_scene.materials.len(), 1);
    assert_eq!(imported_scene.textures.len(), 1);
    assert_eq!(imported_scene.images.len(), 1);
    assert_eq!(imported_scene.samplers.len(), 1);

    let material = &imported_scene.materials[0];
    let texture_ref = material
        .base_color_texture
        .expect("expected base color texture");
    let texture = &imported_scene.textures[texture_ref.texture_index];
    let image = &imported_scene.images[texture.image_index];
    let sampler = &imported_scene.samplers[texture.sampler_index.unwrap()];

    assert_eq!(material.index, 0);
    assert_eq!(material.name.as_deref(), Some("PaintedMaterial"));
    assert_vec4_eq(
        material.base_color_factor,
        Vec4::new(0.5, 0.75, 1.0, 0.5),
        "inline textured base color factor",
    );
    assert_eq!(material.alpha_mode, ImportedAlphaMode::Blend);
    assert_eq!(texture_ref.tex_coord, 0);
    assert_eq!(texture.name.as_deref(), Some("PixelTexture"));
    assert_eq!(image.name.as_deref(), Some("InlinePixel"));
    assert_eq!(image.width, 1);
    assert_eq!(image.height, 1);
    assert_eq!(sampler.name.as_deref(), Some("PixelSampler"));
    assert_eq!(sampler.mag_filter, ImportedMagFilter::Nearest);
    assert_eq!(sampler.min_filter, ImportedMinFilter::LinearMipmapNearest);
    assert_eq!(sampler.wrap_s, ImportedWrapMode::ClampToEdge);
    assert_eq!(sampler.wrap_t, ImportedWrapMode::MirroredRepeat);
}

#[test]
fn test_load_from_path_reads_external_material_image() {
    let imported_scene = load_from_path("material_texture_external.gltf").unwrap();

    assert_eq!(imported_scene.images.len(), 1);
    assert_eq!(imported_scene.images[0].width, 1);
    assert_eq!(imported_scene.images[0].height, 1);
    assert_eq!(
        imported_scene.materials[0].alpha_mode,
        ImportedAlphaMode::Mask
    );
    assert_eq!(imported_scene.materials[0].alpha_cutoff, Some(0.25));
}

#[test]
fn test_load_from_bundle_resolves_external_sidecars() {
    let imported_scene = load_from_bundle(
        "material_texture_external.gltf",
        &[
            "material_texture_external.gltf",
            "vertex_colors.bin",
            "single_pixel.png",
        ],
    )
    .unwrap();

    assert_eq!(imported_scene.images.len(), 1);
    assert_eq!(imported_scene.images[0].width, 1);
    assert_eq!(imported_scene.images[0].height, 1);
    assert_eq!(
        imported_scene.materials[0].alpha_mode,
        ImportedAlphaMode::Mask
    );
}

#[test]
fn test_load_from_bundle_rejects_missing_external_buffer() {
    assert_loader_error_contains(
        load_from_bundle(
            "material_texture_external.gltf",
            &["material_texture_external.gltf"],
        ),
        "Missing uploaded sidecar resource `vertex_colors.bin`",
    );
}

#[test]
fn test_load_from_bundle_uses_fallback_for_missing_external_image() {
    let imported_scene = load_from_bundle(
        "material_texture_external.gltf",
        &["material_texture_external.gltf", "vertex_colors.bin"],
    )
    .unwrap();

    assert_eq!(imported_scene.images.len(), 1);
    assert_eq!(
        imported_scene.images[0].name.as_deref(),
        Some("ExternalPixel")
    );
    assert_eq!(imported_scene.images[0].width, 1);
    assert_eq!(imported_scene.images[0].height, 1);
    assert_eq!(imported_scene.images[0].pixels_rgba8, [255, 255, 255, 255]);
    assert!(
        imported_scene
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic
                .message
                .contains("Missing uploaded sidecar resource `single_pixel.png`")),
        "expected missing image sidecar diagnostic"
    );
}

#[test]
fn test_load_from_path_uses_fallback_for_invalid_material_image() {
    let imported_scene = load_from_path("material_texture_invalid_image.gltf").unwrap();

    assert_eq!(imported_scene.images.len(), 1);
    assert_eq!(
        imported_scene.images[0].name.as_deref(),
        Some("InvalidImage")
    );
    assert_eq!(imported_scene.images[0].width, 1);
    assert_eq!(imported_scene.images[0].height, 1);
    assert_eq!(imported_scene.images[0].pixels_rgba8, [255, 255, 255, 255]);
    assert!(
        imported_scene
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic
                .message
                .contains("Failed to decode image 0 in asset")),
        "expected invalid image diagnostic"
    );
}

#[test]
fn test_load_from_bytes_rejects_relative_external_image() {
    let bytes = std::fs::read(fixture_path("material_texture_external.gltf")).unwrap();

    assert_loader_error_contains(
        load_from_bytes(bytes),
        "cannot be resolved from in-memory glTF bytes",
    );
}

#[test]
fn test_load_from_path_rejects_unsupported_base_color_tex_coord_set() {
    assert_loader_error_contains(
        load_from_path("material_texture_unsupported_texcoord.gltf"),
        "only `TEXCOORD_0` is supported",
    );
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
