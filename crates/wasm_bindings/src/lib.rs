#[cfg(target_arch = "wasm32")]
pub mod bindings;

#[cfg(not(target_arch = "wasm32"))]
pub mod bindings {}
