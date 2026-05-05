use std::ops::Deref;

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Debug, Default, Clone, Hash, Eq, PartialEq)]
pub struct Id {
    value: String,
}

#[wasm_bindgen]
impl Id {
    pub fn new(value: String) -> Self {
        Self { value }
    }

    #[wasm_bindgen(getter)]
    pub fn get_value(&self) -> String {
        self.value.clone()
    }
}

pub trait BaseId {
    fn uuid() -> Self;
    fn get_id(&self) -> &str;
}

impl BaseId for Id {
    fn get_id(&self) -> &str {
        &self.value
    }

    fn uuid() -> Self {
        Self {
            value: uuid::Uuid::new_v4().to_string(),
        }
    }
}

impl Deref for Id {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        &self.value
    }
}
