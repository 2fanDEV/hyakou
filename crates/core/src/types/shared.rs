use glam::Vec3;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Debug, Copy, Clone, PartialEq)]
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
    pub fn from_vec3(vector: Vec3) -> Self {
        Self {
            x: vector.x,
            y: vector.y,
            z: vector.z,
        }
    }

    pub fn to_vec(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct AssetInformation {
    id: String,
    bytes: Vec<u8>,
    name: String,
    pub size: u64,
    pub modified: i32,
}

#[wasm_bindgen]
impl AssetInformation {
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, bytes: Vec<u8>, name: String, size: u64, modified: i32) -> Self {
        Self {
            id,
            bytes,
            name,
            size,
            modified,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
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

#[derive(Debug, Clone)]
pub struct AssetBundleInformation {
    id: String,
    entry_file_name: String,
    files: Vec<AssetInformation>,
}

impl AssetBundleInformation {
    pub fn new(id: String, entry_file_name: String, files: Vec<AssetInformation>) -> Self {
        Self {
            id,
            entry_file_name,
            files,
        }
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn entry_file_name(&self) -> String {
        self.entry_file_name.clone()
    }

    pub fn files(&self) -> Vec<AssetInformation> {
        self.files.clone()
    }
}
