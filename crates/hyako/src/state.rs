use std::{io::Result, sync::Arc};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

#[cfg(target_arch = "wasm32")]
use wgpu::web_sys::HtmlCanvasElement;
#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::PhysicalSize;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowAttributesExtWebSys;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, ElementState, WindowEvent},
    keyboard::PhysicalKey,
    window::{Window, WindowAttributes},
};

use shared::{Shared, shared};

use hyakou_core::{
    components::AssetType,
    events::Event,
    types::{DeltaTime64, mouse_delta::MouseButton},
};

use crate::{
    flow::{CameraController, FlowCommand, FlowController, FlowHandle},
    renderer::SceneRenderer,
};

const SUZANNE_GLTF_BYTES: &[u8] = include_bytes!("../assets/gltf/Suzanne.gltf");
const SUZANNE_BIN_BYTES: &[u8] = include_bytes!("../assets/gltf/Suzanne.bin");
const CUBE_GLTF_BYTES: &[u8] = include_bytes!("../assets/gltf/Cube.gltf");
const CUBE_BIN_BYTES: &[u8] = include_bytes!("../assets/gltf/Cube.bin");

pub struct AppState {
    window: Option<Arc<Window>>,
    #[cfg(target_arch = "wasm32")]
    html_canvas_element: Option<HtmlCanvasElement>,
    flow_controller: FlowController,
    flow_handle: FlowHandle,
    renderer: Shared<Option<Arc<SceneRenderer>>>,
    camera: Shared<Option<Arc<CameraController>>>,
    last_frame_time: Instant,
}

impl AppState {
    const MAX_DELTA_SECONDS: f64 = 0.05;

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Result<Self> {
        let renderer = shared(None);
        let camera = shared(None);
        let (flow_controller, flow_handle) =
            FlowController::new_pair(renderer.clone(), camera.clone());
        Ok(Self {
            window: None,
            flow_controller,
            flow_handle,
            renderer,
            camera,
            last_frame_time: Instant::now(),
        })
    }

    pub fn renderer(&self) -> Shared<Option<Arc<SceneRenderer>>> {
        self.renderer.clone()
    }

    pub fn camera(&self) -> Shared<Option<Arc<CameraController>>> {
        self.camera.clone()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn from_canvas_ref(
        canvas_ref: HtmlCanvasElement,
        upload_status_callback: Shared<Option<js_sys::Function>>,
    ) -> Result<Self> {
        let renderer = shared(None);
        let camera = shared(None);
        let (flow_controller, flow_handle) =
            FlowController::new_pair(renderer.clone(), camera.clone(), upload_status_callback);
        Ok(Self {
            window: None,
            html_canvas_element: Some(canvas_ref),
            flow_controller,
            flow_handle,
            renderer,
            camera,
            last_frame_time: Instant::now(),
        })
    }

    fn get_and_update_last_frame_time(&mut self) -> f64 {
        let now = Instant::now();
        let delta_time = self.get_last_frame_time(now);
        self.last_frame_time = now;
        delta_time
    }

    fn get_last_frame_time(&self, now: Instant) -> DeltaTime64 {
        let delta = now.duration_since(self.last_frame_time);
        delta.as_secs_f64().min(Self::MAX_DELTA_SECONDS)
    }

    fn send_and_drain(&mut self, command: FlowCommand) {
        self.flow_handle.send(command);
        self.flow_controller.drain_commands();
    }
}

impl ApplicationHandler<Event> for AppState {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        #[cfg(not(target_arch = "wasm32"))]
        let window_attributes =
            WindowAttributes::default().with_inner_size(PhysicalSize::new(1920, 1080));

        #[cfg(target_arch = "wasm32")]
        let window_attributes =
            WindowAttributes::default().with_canvas(self.html_canvas_element.clone());

        let window = match event_loop.create_window(window_attributes).map(Arc::new) {
            Ok(window) => window,
            Err(error) => {
                log::error!("Failed to create application window: {error:?}");
                event_loop.exit();
                return;
            }
        };

        self.send_and_drain(FlowCommand::WindowCreated(window.clone()));
        self.send_and_drain(FlowCommand::AssetBundleUploadRequested {
            id: String::from("Suzanne"),
            file_name: String::from("Suzanne.gltf"),
            asset_type: AssetType::NORMAL,
            files: vec![
                (String::from("Suzanne.gltf"), SUZANNE_GLTF_BYTES.to_vec()),
                (String::from("Suzanne.bin"), SUZANNE_BIN_BYTES.to_vec()),
            ],
        });
        self.send_and_drain(FlowCommand::AssetBundleUploadRequested {
            id: String::from("Cube"),
            file_name: String::from("Cube.gltf"),
            asset_type: AssetType::LIGHT,
            files: vec![
                (String::from("Cube.gltf"), CUBE_GLTF_BYTES.to_vec()),
                (String::from("Cube.bin"), CUBE_BIN_BYTES.to_vec()),
            ],
        });
        self.window = Some(window.clone());
        window.request_redraw();
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: Event) {
        match event {
            Event::AnimateCamera(request) => {
                self.send_and_drain(FlowCommand::AnimateCamera(request));
            }
            Event::StopCameraAnimation => {
                self.send_and_drain(FlowCommand::StopCameraAnimation);
            }
            Event::SetCameraMode(mode) => {
                self.send_and_drain(FlowCommand::SetCameraMode(mode));
            }
            Event::AssetUpload(asset_information, asset_type) => {
                self.send_and_drain(FlowCommand::AssetUploadRequested {
                    id: asset_information.id(),
                    file_name: asset_information.name(),
                    asset_type,
                    bytes: asset_information.bytes(),
                });
            }
            Event::AssetBundleUpload(asset_bundle_information, asset_type) => {
                let files = asset_bundle_information
                    .files()
                    .iter()
                    .map(|file_info| (file_info.name(), file_info.bytes()))
                    .collect();
                self.send_and_drain(FlowCommand::AssetBundleUploadRequested {
                    id: asset_bundle_information.id(),
                    file_name: asset_bundle_information.entry_file_name(),
                    asset_type,
                    files,
                });
            }
            Event::WindowResized { width, height } => {
                let dt = self.get_and_update_last_frame_time();
                self.send_and_drain(FlowCommand::HandleResize { dt, width, height });
            }
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let egui_consumed = self.flow_controller.handle_egui_window_event(&event);

        match event {
            WindowEvent::RedrawRequested => {
                let dt = self.get_and_update_last_frame_time();
                self.send_and_drain(FlowCommand::RequestFrame { dt });
            }
            WindowEvent::Resized(size) => {
                let dt = self.get_and_update_last_frame_time();
                self.send_and_drain(FlowCommand::HandleResize {
                    dt,
                    width: size.width as f64,
                    height: size.height as f64,
                });
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if egui_consumed {
                    return;
                }
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    self.send_and_drain(FlowCommand::KeyboardInput {
                        key: key_code,
                        pressed: event.state == ElementState::Pressed,
                    });
                }
            }
            WindowEvent::CursorEntered { .. } => {
                if egui_consumed {
                    return;
                }
                self.send_and_drain(FlowCommand::CursorInWindow { is_inside: true });
            }
            WindowEvent::CursorLeft { .. } => {
                if egui_consumed {
                    return;
                }
                self.send_and_drain(FlowCommand::CursorInWindow { is_inside: false });
            }
            WindowEvent::CursorMoved { position, .. } => {
                if egui_consumed {
                    return;
                }
                self.send_and_drain(FlowCommand::CursorMoved {
                    x: position.x,
                    y: position.y,
                });
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                let dt = self.get_and_update_last_frame_time() as f32;
                self.send_and_drain(FlowCommand::MouseMotion {
                    dx: delta.0,
                    dy: delta.1,
                    dt,
                });
            }
            DeviceEvent::Button { button, state } => {
                let mouse_button = match button {
                    0 => MouseButton::Left,
                    1 => MouseButton::Right,
                    2 => MouseButton::Middle,
                    _ => return,
                };
                self.send_and_drain(FlowCommand::MouseButtonInput {
                    button: mouse_button,
                    pressed: state == ElementState::Pressed,
                });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_accurate_calculation() {
        let mut state = AppState::new().unwrap();

        let delta_time = state.get_and_update_last_frame_time();
        assert!(delta_time.is_finite());
    }

    #[test]
    fn test_delta_is_not_forced_to_maximum() {
        let state = AppState::new().unwrap();

        let last_frame_time_instant = state.last_frame_time;
        let delta_time = state.get_last_frame_time(last_frame_time_instant);
        assert_eq!(delta_time, 0.0);
    }

    #[test]
    fn test_delta_is_capped_to_maximum() {
        let state = AppState::new().unwrap();

        let now = state.last_frame_time + Duration::from_secs(1);
        let delta_time = state.get_last_frame_time(now);
        assert_eq!(delta_time, AppState::MAX_DELTA_SECONDS);
    }
}
