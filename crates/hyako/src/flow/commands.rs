use std::sync::Arc;

use crate::gpu::glTF::ImportedScene;
use hyakou_core::{
    components::{LightType, camera::data_structures::CameraAnimationRequest},
    types::mouse_delta::MouseButton,
};
use winit::{keyboard::KeyCode, window::Window};

pub enum RendererCommand {
    WindowCreated(Arc<Window>),
    AnimateCamera(CameraAnimationRequest),
    StopCameraAnimation,
    CursorInWindow {
        is_inside: bool,
    },
    CursorMoved {
        x: f64,
        y: f64,
    },
    KeyboardInput {
        key: KeyCode,
        pressed: bool,
    },
    MouseMotion {
        dx: f64,
        dy: f64,
        dt: f32,
    },
    MouseButton {
        button: MouseButton,
        pressed: bool,
    },
    AssetUploadRequested {
        id: String,
        file_name: String,
        asset_type: LightType,
        bytes: Vec<u8>,
    },
    AssetBundleUploadRequested {
        id: String,
        file_name: String,
        asset_type: LightType,
        files: Vec<(String, Vec<u8>)>,
    },
    ApplyParsedAsset {
        id: String,
        file_name: String,
        asset_type: LightType,
        imported_scene: ImportedScene,
    },
    AssetUploadFailed {
        id: String,
        file_name: String,
        error: String,
    },
    Redraw {
        dt: f64,
    },
    Resize {
        dt: f64,
        height: f64,
        width: f64,
    },
}
