use glam::Vec3;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Debug)]
pub struct Coordinates3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[wasm_bindgen]
impl Coordinates3 {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl Coordinates3 {
    pub fn to_vec(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

#[wasm_bindgen]
pub struct AssetInformation {
    bytes: Vec<u8>,
    name: String,
    pub size: u64,
    pub modified: i32,
}

#[wasm_bindgen]
impl AssetInformation {
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: Vec<u8>, name: String, size: u64, modified: i32) -> Self {
        Self {
            bytes,
            name,
            size,
            modified,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }
}
