use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Debug)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[wasm_bindgen]
pub struct FileInformation {
    path: String,
    name: String,
    pub size: u64,
    pub modified: i32,
}

#[wasm_bindgen]
impl FileInformation {
    #[wasm_bindgen(constructor)]
    pub fn new(path: String, name: String, size: u64, modified: i32) -> Self {
        Self {
            path,
            name,
            size,
            modified,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn path(&self) -> String {
        self.path.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }
}
