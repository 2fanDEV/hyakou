use std::sync::Arc;

use hyakou_core::{
    components::{LightType, mesh_node::MeshNode},
    types::{mouse_delta::MouseButton, shared::Coordinates},
};
use winit::{keyboard::KeyCode, window::Window};

pub enum RendererCommand {
    WindowCreated(Arc<Window>),
    SetCoords(Coordinates),
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
        asset_type: LightType,
        bytes: Vec<u8>,
    },
    ApplyParsedAsset {
        id: String,
        asset_type: LightType,
        mesh_nodes: Vec<MeshNode>,
    },
    AssetUploadFailed {
        id: String,
        error: String,
    },
    Redraw {
        dt: f64,
    },
}
