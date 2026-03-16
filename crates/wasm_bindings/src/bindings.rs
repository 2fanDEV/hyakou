use hyako::state::AppState;
use hyakou_core::events::Event;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::HtmlCanvasElement;
use winit::{event_loop::EventLoop, platform::web::EventLoopExtWebSys};

#[wasm_bindgen]
pub fn start(canvas_ref: HtmlCanvasElement) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let event_loop = match EventLoop::<Event>::with_user_event().build() {
        Ok(event_loop) => event_loop,
        Err(error) => return Err(JsValue::from_str(&error.to_string())),
    };
    log::info!("Event loop initialized!");
    let app_state = match AppState::from_canvas_ref(canvas_ref) {
        Ok(app_state) => app_state,
        Err(error) => return Err(JsValue::from_str(&error.to_string())),
    };
    event_loop.spawn_app(app_state);

    Ok(())
}
