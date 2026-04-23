use hyakou_core::{
    components::camera::{
        camera::Camera,
        data_structures::{
            CameraAnimationEasing, CameraAnimationRequest, CameraAnimationStateSnapshot,
        },
    },
    types::{base::Id, shared::Coordinates3},
};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[cfg(target_arch = "wasm32")]
pub mod bindings;

#[cfg(not(target_arch = "wasm32"))]
pub mod bindings {}

pub use hyakou_core::types::upload_status::UploadStatusEvent;
#[wasm_bindgen(typescript_custom_section)]
const CAMERA_ANIMATION_TYPES: &str = r#"
export type CameraAnimationEasingName = "linear" | "ease-in" | "ease-out" | "ease-in-out";
"#;

#[wasm_bindgen]
pub struct CameraAnimationOptions {
    duration_ms: Option<f64>,
    easing: CameraAnimationEasing,
}

#[wasm_bindgen]
impl CameraAnimationOptions {
    #[wasm_bindgen(constructor)]
    pub fn new(duration_ms: Option<f64>, easing: Option<String>) -> Result<Self, JsValue> {
        Self::try_new(duration_ms, easing.as_deref()).map_err(|error| JsValue::from_str(&error))
    }

    #[wasm_bindgen(getter, js_name = durationMs)]
    pub fn duration_ms(&self) -> Option<f64> {
        self.duration_ms
    }

    #[wasm_bindgen(getter)]
    pub fn easing(&self) -> String {
        self.easing.as_str().to_string()
    }
}

impl CameraAnimationOptions {
    fn try_new(duration_ms: Option<f64>, easing: Option<&str>) -> Result<Self, String> {
        Ok(Self {
            duration_ms,
            easing: parse_camera_animation_easing(easing)?,
        })
    }

    pub fn to_request(&self, target_coords: Coordinates3) -> CameraAnimationRequest {
        CameraAnimationRequest::new(
            target_coords,
            self.duration_ms.map(|value| (value / 1000.0) as f32),
            self.easing,
        )
    }
}

impl Default for CameraAnimationOptions {
    fn default() -> Self {
        Self {
            duration_ms: None,
            easing: CameraAnimationEasing::default(),
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct CameraAnimationStateDO {
    pub active: bool,
    pub progress: f32,
    #[wasm_bindgen(js_name = durationMs)]
    pub duration_ms: f64,
    #[wasm_bindgen(js_name = elapsedMs)]
    pub elapsed_ms: f64,
    #[wasm_bindgen(js_name = targetCoords)]
    pub target_coords: Coordinates3,
    pub easing: String,
}

impl CameraAnimationStateDO {
    pub fn from_snapshot(snapshot: CameraAnimationStateSnapshot) -> Self {
        Self {
            active: snapshot.active,
            progress: snapshot.progress,
            duration_ms: f64::from(snapshot.duration_seconds) * 1000.0,
            elapsed_ms: f64::from(snapshot.elapsed_seconds) * 1000.0,
            target_coords: snapshot.target_coords,
            easing: snapshot.easing.as_str().to_string(),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct CameraDO {
    id: Id,
    pub eye: Coordinates3,
    pub target: Coordinates3,
    pub up: Coordinates3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub speed: f32,
    pub sensitivity: f32,
    pub smoothing_factor: f32,
}

impl CameraDO {
    pub fn from_camera(camera: &Camera) -> CameraDO {
        Self {
            id: camera.id.clone(),
            eye: Coordinates3::from_vec3(camera.eye),
            target: Coordinates3::from_vec3(camera.target),
            up: Coordinates3::from_vec3(camera.up),
            aspect: camera.aspect,
            fovy: camera.fovy,
            znear: camera.znear,
            zfar: camera.zfar,
            speed: camera.speed,
            sensitivity: camera.sensitivity,
            smoothing_factor: camera.smoothing_factor,
        }
    }
}

#[wasm_bindgen]
impl CameraDO {
    #[wasm_bindgen(getter)]
    pub fn get_camera_id(&self) -> Id {
        self.id.clone()
    }
}

fn parse_camera_animation_easing(value: Option<&str>) -> Result<CameraAnimationEasing, String> {
    match value.unwrap_or("linear") {
        "linear" => Ok(CameraAnimationEasing::Linear),
        "ease-in" => Ok(CameraAnimationEasing::EaseIn),
        "ease-out" => Ok(CameraAnimationEasing::EaseOut),
        "ease-in-out" => Ok(CameraAnimationEasing::EaseInOut),
        invalid => Err(format!("Unsupported camera animation easing `{invalid}`")),
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;
    use hyakou_core::types::camera::{Pitch, Yaw};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    fn create_test_camera() -> Camera {
        Camera::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::Y,
            16.0 / 9.0,
            45.0_f32.to_radians(),
            0.1,
            100.0,
            Yaw::new(0.0),
            Pitch::new(0.0),
            20.0,
            0.5,
            0.25,
        )
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn test_camera_animation_options_to_request_converts_milliseconds_to_seconds() {
        let options =
            CameraAnimationOptions::new(Some(1500.0), Some("ease-out".to_string())).unwrap();
        let request = options.to_request(Coordinates3::new(9.0, 8.0, 7.0));

        assert_eq!(request.target_coords(), Coordinates3::new(9.0, 8.0, 7.0));
        assert!(
            (request.resolve_duration_seconds(Coordinates3::new(0.0, 0.0, 0.0), 20.0) - 1.5).abs()
                < 0.0001
        );
        assert_eq!(request.easing(), CameraAnimationEasing::EaseOut);
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn test_camera_animation_options_rejects_unknown_easing() {
        let actual = CameraAnimationOptions::try_new(None, Some("bounce"));

        assert!(actual.is_err());
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn test_camera_animation_state_data_object_uses_snapshot_values() {
        let snapshot = CameraAnimationStateSnapshot {
            active: true,
            progress: 0.25,
            duration_seconds: 2.0,
            elapsed_seconds: 0.5,
            target_coords: Coordinates3::new(7.0, 8.0, 9.0),
            easing: CameraAnimationEasing::EaseInOut,
        };

        let state = CameraAnimationStateDO::from_snapshot(snapshot);

        assert!(state.active);
        assert!((state.progress - 0.25).abs() < 0.0001);
        assert_eq!(state.duration_ms, 2000.0);
        assert_eq!(state.elapsed_ms, 500.0);
        assert_eq!(state.target_coords, Coordinates3::new(7.0, 8.0, 9.0));
        assert_eq!(state.easing, "ease-in-out");
    }

    #[wasm_bindgen_test(unsupported = test)]
    fn test_camera_data_object_maps_camera_fields() {
        let camera = create_test_camera();

        let camera_do = CameraDO::from_camera(&camera);

        assert_eq!(camera_do.eye, Coordinates3::new(1.0, 2.0, 3.0));
        assert_eq!(camera_do.target, Coordinates3::new(4.0, 5.0, 6.0));
        assert_eq!(camera_do.up, Coordinates3::new(0.0, 1.0, 0.0));
        assert_eq!(camera_do.speed, 20.0);
        assert_eq!(camera_do.sensitivity, 0.5);
        assert_eq!(camera_do.smoothing_factor, 0.25);
    }
}
