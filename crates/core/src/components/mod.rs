use wasm_bindgen::prelude::wasm_bindgen;

pub mod camera;
pub mod light;
pub mod mesh_node;

#[wasm_bindgen]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    LIGHT,
    NO_LIGHT,
}
