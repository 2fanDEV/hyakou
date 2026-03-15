use hyako::state::AppState;
use log::debug;
use wasm_bindgen::{JsError, prelude::wasm_bindgen};
use wgpu::web_sys::HtmlCanvasElement;
use winit::{event_loop::EventLoop, platform::web::EventLoopExtWebSys};

#[wasm_bindgen]
pub fn start(canvas_ref: HtmlCanvasElement) -> Result<(), JsError> {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    debug!("start(): wasm entry called");

    let app_state = AppState::from_canvas_ref(canvas_ref)
        .map_err(|e| JsError::new(&format!("start(): AppState::new failed: {e}")))?;
    debug!("start(): AppState created");

    let event_loop = EventLoop::new()
        .map_err(|e| JsError::new(&format!("start(): EventLoop::new failed: {e}")))?;
    debug!("start(): EventLoop created, calling spawn_app");

    event_loop.spawn_app(app_state);
    debug!("start(): spawn_app returned");

    Ok(())
}

#[wasm_bindgen]
pub fn full_start() -> String {
    return "Initialized".to_string();
}
