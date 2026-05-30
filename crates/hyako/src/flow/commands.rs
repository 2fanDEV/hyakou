use std::sync::Arc;

use crate::{
    flow::CameraController,
    gpu::glTF::ImportedScene, renderer::SceneRenderer,
};
use hyakou_core::{
    components::{
        AssetType,
        camera::data_structures::{CameraAnimationRequest, CameraMode},
    },
    selection::structure::SelectionScope,
    types::mouse_delta::MouseButton,
};

use winit::{keyboard::KeyCode, window::Window};

pub enum FlowCommand {
    WindowCreated(Arc<Window>),
    RendererInitialized {
        renderer: SceneRenderer,
        camera_controller: CameraController,
    },
    AnimateCamera(CameraAnimationRequest),
    StopCameraAnimation,
    SetCameraMode(CameraMode),
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
        asset_type: AssetType,
        bytes: Vec<u8>,
    },
    AssetBundleUploadRequested {
        id: String,
        file_name: String,
        asset_type: AssetType,
        files: Vec<(String, Vec<u8>)>,
    },
    ApplyParsedAsset {
        id: String,
        file_name: String,
        asset_type: AssetType,
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
    SelectAtScreenPoint {
        x: f32,
        y: f32,
        scope: SelectionScope,
    },
}
