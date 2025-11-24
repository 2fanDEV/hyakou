pub mod camera;
pub mod glTF;
pub mod mesh_node;
pub mod texture;
pub mod render_pipeline;
pub mod light;
pub mod asset_manager;
pub mod render_mesh;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LightType {
    LIGHT,
    NO_LIGHT
}