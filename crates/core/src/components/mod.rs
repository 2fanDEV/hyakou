pub mod camera;
pub mod light;
pub mod mesh_node;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LightType {
    LIGHT,
    NO_LIGHT,
}
