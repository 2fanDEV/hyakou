pub mod renderer;
pub mod state;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    // Set panic hook for better error messages in browser console
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    // Initialize logging for WASM
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");

    log::info!("Hyakou WASM initialized");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn run() -> Result<(), JsValue> {
    use winit::event_loop::EventLoop;
    use winit::platform::web::EventLoopExtWebSys;

    let event_loop = EventLoop::builder().build()
        .map_err(|e| JsValue::from_str(&format!("Failed to create event loop: {}", e)))?;

    let mut app_state = state::AppState::new();

    // Use spawn_app for WASM compatibility
    event_loop.spawn_app(app_state);

    Ok(())
}