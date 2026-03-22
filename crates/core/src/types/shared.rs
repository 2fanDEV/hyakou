use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Debug)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
    pub z: i32,
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
