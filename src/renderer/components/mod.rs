pub mod camera;
pub mod glTF;
pub mod light;
pub mod mesh_node;
pub mod render_mesh;
pub mod render_pipeline;
pub mod texture;
pub mod transform;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LightType {
    LIGHT,
    NO_LIGHT,
}
