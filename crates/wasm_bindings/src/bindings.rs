use hyako::state::AppState;
use hyakou_core::{
    events::Event,
    types::shared::{AssetInformation, Coordinates3},
};
use log::Record;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::HtmlCanvasElement;
use winit::{
    event_loop::{EventLoop, EventLoopProxy},
    platform::web::EventLoopExtWebSys,
};

#[wasm_bindgen]
pub struct Hyako {
    app_state: Option<AppState>,
    event_loop: Option<EventLoop<Event>>,
    event_loop_proxy: EventLoopProxy<Event>,
}

#[wasm_bindgen]
impl Hyako {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_ref: HtmlCanvasElement) -> Result<Hyako, JsValue> {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Info);
        let event_loop = match EventLoop::<Event>::with_user_event().build() {
            Ok(event_loop) => event_loop,
            Err(error) => return Err(JsValue::from_str(&error.to_string())),
        };
        log::info!("Event loop initialized!");
        let app_state = match AppState::from_canvas_ref(canvas_ref) {
            Ok(app_state) => app_state,
            Err(error) => return Err(JsValue::from_str(&error.to_string())),
        };
        let event_loop_proxy = event_loop.create_proxy();
        Ok(Hyako {
            app_state: Some(app_state),
            event_loop: Some(event_loop),
            event_loop_proxy,
        })
    }

    #[wasm_bindgen]
    pub fn start_rendering(&mut self) -> Result<(), JsValue> {
        let event_loop = self
            .event_loop
            .take()
            .ok_or_else(|| JsValue::from_str("Renderer event loop already running or missing"))?;
        let app_state = self
            .app_state
            .take()
            .ok_or_else(|| JsValue::from_str("Renderer app state already consumed or missing"))?;

        event_loop.spawn_app(app_state);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn set_coords(&self, coordinates: Coordinates3) -> Result<(), JsValue> {
        self.send_event(Event::SetCoords(coordinates))
    }

    #[wasm_bindgen]
    pub fn upload_file(&self, file: AssetInformation) -> Result<(), JsValue> {
        self.send_event(Event::AssetUpload(file))
    }

    fn send_event(&self, event: Event) -> Result<(), JsValue> {
        match self.event_loop_proxy.send_event(event) {
            Ok(_) => Ok(()),
            Err(msg) => Err(JsValue::from_str(&msg.to_string())),
        }
    }
}
