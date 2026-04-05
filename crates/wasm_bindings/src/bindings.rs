use hyako::{renderer::Renderer, state::AppState};
use hyakou_core::{
    Shared, SharedAccess,
    events::Event,
    types::shared::{AssetInformation, Coordinates3},
};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;
use winit::event_loop::{EventLoop, EventLoopProxy};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

use crate::{CameraAnimationOptions, CameraAnimationStateDO, CameraDO};

#[wasm_bindgen]
pub struct Hyako {
    app_state: Option<AppState>,
    renderer: Shared<Option<Renderer>>,
    event_loop: Option<EventLoop<Event>>,
    event_loop_proxy: EventLoopProxy<Event>,
}

#[wasm_bindgen]
impl Hyako {
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_ref: HtmlCanvasElement) -> Result<Hyako, JsValue> {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Debug);
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
        let renderer = app_state.get_renderer();
        Ok(Hyako {
            app_state: Some(app_state),
            renderer,
            event_loop: Some(event_loop),
            event_loop_proxy,
        })
    }

    #[cfg(target_arch = "wasm32")]
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
    pub fn animate_camera_to(
        &self,
        coordinates: Coordinates3,
        options: Option<CameraAnimationOptions>,
    ) -> Result<(), JsValue> {
        let request = match options {
            Some(options) => options.to_request(coordinates),
            None => CameraAnimationOptions::default().to_request(coordinates),
        };

        self.send_event(Event::AnimateCamera(request))
    }

    #[wasm_bindgen(js_name = set_coords)]
    pub fn set_coords(&self, coordinates: Coordinates3) -> Result<(), JsValue> {
        self.animate_camera_to(coordinates, None)
    }

    #[wasm_bindgen]
    pub fn stop_camera_animation(&self) -> Result<(), JsValue> {
        self.send_event(Event::StopCameraAnimation)
    }

    #[wasm_bindgen]
    pub fn upload_file(&self, file: AssetInformation) -> Result<(), JsValue> {
        self.send_event(Event::AssetUpload(file))
    }

    #[wasm_bindgen]
    pub fn get_camera(&self) -> Result<CameraDO, JsValue> {
        self.renderer
            .try_read_shared(|renderer| match renderer {
                Some(r) => Ok(CameraDO::from_camera(&r.camera)),
                None => Err(JsValue::from_str("Renderer missing or not initialized")),
            })
            .unwrap()
    }

    #[wasm_bindgen]
    pub fn get_camera_animation_state(&self) -> Result<CameraAnimationStateDO, JsValue> {
        self.renderer
            .try_read_shared(|renderer| match renderer {
                Some(renderer) => Ok(CameraAnimationStateDO::from_snapshot(
                    renderer
                        .camera_handler
                        .state
                        .camera_animation_state(&renderer.camera),
                )),
                None => Err(JsValue::from_str("Renderer missing or not initialized")),
            })
            .unwrap()
    }

    #[wasm_bindgen]
    pub fn is_camera_animating(&self) -> Result<bool, JsValue> {
        self.get_camera_animation_state().map(|state| state.active)
    }

    #[wasm_bindgen]
    pub fn resize(&mut self, width: f64, height: f64) -> Result<(), JsValue> {
        self.send_event(Event::Resize(width, height))
    }

    fn send_event(&self, event: Event) -> Result<(), JsValue> {
        match self.event_loop_proxy.send_event(event) {
            Ok(_) => Ok(()),
            Err(msg) => Err(JsValue::from_str(&msg.to_string())),
        }
    }
}
