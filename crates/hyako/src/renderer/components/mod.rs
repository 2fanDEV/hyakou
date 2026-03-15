pub mod camera;
#[allow(non_snake_case)]
pub mod glTF;
pub mod light;
pub mod mesh_node;
pub mod model_matrix;
pub mod render_mesh;
pub mod render_pipeline;
pub mod texture;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LightType {
    LIGHT,
    NO_LIGHT,
}
