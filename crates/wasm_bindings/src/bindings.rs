use hyako::{renderer::Renderer, state::AppState};
use hyakou_core::{
    Shared, SharedAccess,
    components::{LightType, camera::data_structures::CameraMode},
    events::Event,
    shared,
    types::shared::{AssetBundleInformation, AssetInformation, Coordinates3},
};
use js_sys::{Array, BigInt, Reflect, Uint8Array};
use strum::VariantArray;
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
    upload_status_callback: Shared<Option<js_sys::Function>>,
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
        let upload_status_callback: Shared<Option<js_sys::Function>> = shared(None);
        let app_state = match AppState::from_canvas_ref(canvas_ref, upload_status_callback.clone())
        {
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
            upload_status_callback,
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
    pub fn upload_file(
        &self,
        file: AssetInformation,
        light_type: Option<LightType>,
    ) -> Result<(), JsValue> {
        let light_type = light_type.unwrap_or(LightType::LIGHT);
        self.send_event(Event::AssetUpload(file, light_type))
    }

    #[wasm_bindgen]
    pub fn upload_asset_bundle(
        &self,
        id: String,
        entry_file_name: String,
        files: Array,
        light_type: Option<LightType>,
    ) -> Result<(), JsValue> {
        let light_type = light_type.unwrap_or(LightType::LIGHT);
        let files = files
            .iter()
            .map(asset_information_from_js_value)
            .collect::<Result<Vec<_>, _>>()?;

        self.send_event(Event::AssetBundleUpload(
            AssetBundleInformation::new(id, entry_file_name, files),
            light_type,
        ))
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
    pub fn get_camera_modes(&self) -> Result<Vec<CameraMode>, JsValue> {
        Ok(CameraMode::VARIANTS.to_vec())
    }

    #[wasm_bindgen]
    pub fn set_camera_mode(&self, mode: CameraMode) -> Result<(), JsValue> {
        self.renderer
            .try_write_shared(|renderer| match renderer {
                Some(rend) => Ok(rend.camera_handler.set_mode(mode)),
                None => Err(JsValue::from_str("Renderer missing or not initialized")),
            })
            .unwrap()
    }

    #[wasm_bindgen]
    pub fn resize(&mut self, width: f64, height: f64) -> Result<(), JsValue> {
        self.send_event(Event::Resize(width, height))
    }

    #[wasm_bindgen]
    pub fn is_renderer_ready(&mut self) -> Result<bool, JsValue> {
        Ok(self.renderer.try_read_shared(|rnd| rnd.is_some()).unwrap())
    }

    #[wasm_bindgen(js_name = setUploadStatusListener)]
    pub fn set_upload_status_listener(&self, callback: js_sys::Function) {
        let _ = self
            .upload_status_callback
            .try_write_shared(|slot| *slot = Some(callback));
    }

    fn send_event(&self, event: Event) -> Result<(), JsValue> {
        match self.event_loop_proxy.send_event(event) {
            Ok(_) => Ok(()),
            Err(msg) => Err(JsValue::from_str(&msg.to_string())),
        }
    }
}

fn asset_information_from_js_value(value: JsValue) -> Result<AssetInformation, JsValue> {
    let id = js_string_property(&value, "id")?;
    let name = js_string_property(&value, "name")?;
    let size = js_bigint_property(&value, "size")?;
    let modified = js_number_property(&value, "modified")? as i32;
    let bytes = Uint8Array::new(&js_property(&value, "bytes")?).to_vec();

    Ok(AssetInformation::new(id, bytes, name, size, modified))
}

fn js_property(value: &JsValue, property: &str) -> Result<JsValue, JsValue> {
    Reflect::get(value, &JsValue::from_str(property))
        .map_err(|_| JsValue::from_str(&format!("Missing `{property}` on bundle file")))
}

fn js_string_property(value: &JsValue, property: &str) -> Result<String, JsValue> {
    js_property(value, property)?
        .as_string()
        .ok_or_else(|| JsValue::from_str(&format!("`{property}` must be a string")))
}

fn js_number_property(value: &JsValue, property: &str) -> Result<f64, JsValue> {
    js_property(value, property)?
        .as_f64()
        .ok_or_else(|| JsValue::from_str(&format!("`{property}` must be a number")))
}

fn js_bigint_property(value: &JsValue, property: &str) -> Result<u64, JsValue> {
    let bigint = BigInt::from(js_property(value, property)?);
    let string_value = bigint.to_string(10)?.as_string().ok_or_else(|| {
        JsValue::from_str(&format!("`{property}` must be a bigint-compatible value"))
    })?;

    string_value
        .parse()
        .map_err(|_| JsValue::from_str(&format!("`{property}` must be a bigint-compatible value")))
}
