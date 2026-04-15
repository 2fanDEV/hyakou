use hyako_wasm_bindings::CameraAnimationOptions;
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
fn camera_animation_options_accepts_known_easing() {
    let options = CameraAnimationOptions::new(Some(1200.0), Some("ease-in-out".to_string()));

    assert!(options.is_ok());
}
