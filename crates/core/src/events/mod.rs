use wasm_bindgen::prelude::wasm_bindgen;

use crate::types::shared::Coordinates;

#[derive(Debug)]
pub enum Event {
    SetCoords(Coordinates),
    UploadFile(String),
}
