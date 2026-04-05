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

use hyakou_core::{
    Shared,
    components::LightType,
    events::Event,
    types::{DeltaTime64, mouse_delta::MouseButton},
};

use crate::{
    flow::{FlowController, FlowHandle, RendererCommand},
    renderer::Renderer,
};

pub struct AppState {
    window: Option<Arc<Window>>,
    #[cfg(target_arch = "wasm32")]
    html_canvas_element: Option<HtmlCanvasElement>,
    flow_controller: FlowController,
    flow_handle: FlowHandle,
    last_frame_time: Instant,
}

impl AppState {
    const MIN_TIME_IN_SECONDS: f64 = 0.05;

    pub fn new() -> Result<Self> {
        let (flow_controller, flow_handle) = FlowController::new_pair();
        Ok(Self {
            window: None,
            #[cfg(target_arch = "wasm32")]
            html_canvas_element: None,
            flow_controller,
            flow_handle,
            last_frame_time: Instant::now(),
        })
    }

    pub fn get_renderer(&self) -> Shared<Option<Renderer>> {
        self.flow_controller.get_renderer()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn from_canvas_ref(canvas_ref: HtmlCanvasElement) -> Result<Self> {
        let (flow_controller, flow_handle) = FlowController::new_pair();
        Ok(Self {
            window: None,
            html_canvas_element: Some(canvas_ref),
            flow_controller,
            flow_handle,
            last_frame_time: Instant::now(),
        })
    }

    fn get_and_update_last_frame_time(&mut self) -> f64 {
        let now = Instant::now();
        let delta_time = self.get_last_frame_time(now);
        self.last_frame_time = now;
        delta_time
    }

    fn get_last_frame_time(&mut self, now: Instant) -> DeltaTime64 {
        let delta = now.duration_since(self.last_frame_time);
        delta.as_secs_f64().min(Self::MIN_TIME_IN_SECONDS)
    }

    fn send_and_drain(&mut self, command: RendererCommand) {
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

        let window = event_loop
            .create_window(window_attributes)
            .map(Arc::new)
            .unwrap();

        self.send_and_drain(RendererCommand::WindowCreated(window.clone()));

        self.window = Some(window.clone());
        window.request_redraw();
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: Event) {
        match event {
            Event::AnimateCamera(request) => {
                self.send_and_drain(RendererCommand::AnimateCamera(request));
            }
            Event::StopCameraAnimation => {
                self.send_and_drain(RendererCommand::StopCameraAnimation);
            }
            Event::AssetUpload(asset_information) => {
                self.send_and_drain(RendererCommand::AssetUploadRequested {
                    id: asset_information.name(),
                    asset_type: LightType::LIGHT,
                    bytes: asset_information.bytes(),
                });
            }
            Event::Resize(width, height) => {
                let dt = self.get_and_update_last_frame_time();
                self.send_and_drain(RendererCommand::Resize { dt, width, height });
            }
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                let delta = self.get_and_update_last_frame_time();
                self.send_and_drain(RendererCommand::Redraw { dt: delta });
            }
            WindowEvent::CursorEntered { .. } => {
                self.send_and_drain(RendererCommand::CursorInWindow { is_inside: true });
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.send_and_drain(RendererCommand::CursorMoved {
                    x: position.x,
                    y: position.y,
                });
            }
            WindowEvent::CursorLeft { .. } => {
                self.send_and_drain(RendererCommand::CursorInWindow { is_inside: false });
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let PhysicalKey::Code(key) = event.physical_key else {
                    return;
                };
                self.send_and_drain(RendererCommand::KeyboardInput {
                    key,
                    pressed: event.state == ElementState::Pressed,
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
                let dt = self.get_last_frame_time(Instant::now()) as f32;
                self.send_and_drain(RendererCommand::MouseMotion {
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

                self.send_and_drain(RendererCommand::MouseButton {
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
    use std::{thread::sleep, time::Duration};

    use super::*;

    fn setup() -> AppState {
        AppState::new().unwrap()
    }

    #[test]
    fn test_clamping_strategy() {
        let mut state = setup();
        let actual = state.get_and_update_last_frame_time();
        assert!(actual > 0.0);
        assert!(actual <= AppState::MIN_TIME_IN_SECONDS);
        sleep(Duration::from_secs(1));
        let actual = state.get_and_update_last_frame_time();
        assert!(actual <= AppState::MIN_TIME_IN_SECONDS);
    }

    #[test]
    fn test_accurate_calculation() {
        let mut state = setup();
        state.get_and_update_last_frame_time();
        sleep(Duration::from_millis(16));
        let second_delta = state.get_and_update_last_frame_time();
        assert!(second_delta >= 0.015 && second_delta <= AppState::MIN_TIME_IN_SECONDS);
    }
}
