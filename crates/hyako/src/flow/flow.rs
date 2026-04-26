use std::sync::mpsc::{Receiver, Sender, channel};

use hyakou_core::Shared;
use log::{debug, warn};

use crate::{
    flow::{
        AssetUploadController, FrameComposer, InputController, RendererCommand,
        RendererLifecycleController,
    },
    renderer::Renderer,
};

pub struct FlowController {
    rx: Receiver<RendererCommand>,
    renderer_lifecycle_controller: RendererLifecycleController,
    frame_composer: FrameComposer,
    input_controller: InputController,
    asset_upload_controller: AssetUploadController,
}

#[derive(Clone)]
pub struct FlowHandle {
    tx: Sender<RendererCommand>,
}

impl FlowController {
    const MAX_COMMANDS_PER_TICK: usize = 128;

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_pair() -> (Self, FlowHandle) {
        let (tx, rx) = channel::<RendererCommand>();
        let controller = Self {
            rx,
            renderer_lifecycle_controller: RendererLifecycleController::new(),
            frame_composer: FrameComposer::new(),
            input_controller: InputController::new(),
            asset_upload_controller: AssetUploadController::new(tx.clone()),
        };

        (controller, FlowHandle::new(tx))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new_pair(
        upload_status_callback: Shared<Option<js_sys::Function>>,
    ) -> (Self, FlowHandle) {
        let (tx, rx) = channel::<RendererCommand>();
        let controller = Self {
            rx,
            renderer_lifecycle_controller: RendererLifecycleController::new(),
            frame_composer: FrameComposer::new(),
            input_controller: InputController::new(),
            asset_upload_controller: AssetUploadController::new(tx.clone(), upload_status_callback),
        };

        (controller, FlowHandle::new(tx))
    }
    pub fn get_renderer(&self) -> Shared<Option<Renderer>> {
        self.renderer_lifecycle_controller.renderer()
    }

    pub fn handle_egui_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.renderer_lifecycle_controller
            .handle_egui_window_event(event)
    }

    pub fn drain_commands(&mut self) {
        for _ in 0..Self::MAX_COMMANDS_PER_TICK {
            let command = match self.rx.try_recv() {
                Ok(command) => command,
                Err(_) => return,
            };

            self.handle_command(command);
        }

        warn!(
            "FlowController reached max commands per tick; remaining commands will be handled next frame"
        );
    }

    fn handle_command(&mut self, command: RendererCommand) {
        match command {
            RendererCommand::WindowCreated(window) => self
                .renderer_lifecycle_controller
                .handle_window_created(window),
            RendererCommand::AnimateCamera(request) => {
                self.renderer_lifecycle_controller.animate_camera(request)
            }
            RendererCommand::StopCameraAnimation => {
                self.renderer_lifecycle_controller.stop_camera_animation()
            }
            RendererCommand::CursorInWindow { is_inside } => {
                self.input_controller.handle_cursor_in_window(is_inside)
            }
            RendererCommand::CursorMoved { x, y } => {
                self.input_controller.handle_cursor_moved(x, y);
            }
            RendererCommand::KeyboardInput { key, pressed } => {
                let renderer = self.renderer_lifecycle_controller.renderer();
                self.input_controller
                    .handle_keyboard_input(&renderer, key, pressed);
            }
            RendererCommand::MouseMotion { dx, dy, dt } => {
                let renderer = self.renderer_lifecycle_controller.renderer();
                self.input_controller
                    .handle_mouse_motion(&renderer, dx, dy, dt);
            }
            RendererCommand::MouseButton { button, pressed } => {
                let renderer = self.renderer_lifecycle_controller.renderer();
                self.input_controller.handle_mouse_button(
                    &renderer,
                    self.renderer_lifecycle_controller.window(),
                    button,
                    pressed,
                );
            }
            RendererCommand::AssetUploadRequested {
                id,
                file_name,
                asset_type,
                bytes,
            } => self
                .asset_upload_controller
                .handle_asset_upload_requested(id, file_name, asset_type, bytes),
            RendererCommand::AssetBundleUploadRequested {
                id,
                file_name,
                asset_type,
                files,
            } => self
                .asset_upload_controller
                .handle_asset_bundle_upload_requested(id, file_name, asset_type, files),
            RendererCommand::ApplyParsedAsset {
                id,
                file_name,
                asset_type,
                imported_scene,
            } => self.asset_upload_controller.handle_apply_parsed_asset(
                &self.renderer_lifecycle_controller.renderer(),
                id,
                file_name,
                asset_type,
                imported_scene,
            ),
            RendererCommand::AssetUploadFailed {
                id,
                file_name,
                error,
            } => self
                .asset_upload_controller
                .handle_asset_upload_failed(id, file_name, error),
            RendererCommand::Redraw { dt } => self
                .renderer_lifecycle_controller
                .render_frame(&mut self.frame_composer, dt),
            RendererCommand::Resize { dt, width, height } => {
                self.renderer_lifecycle_controller
                    .handle_resize(width, height);
                self.renderer_lifecycle_controller
                    .render_frame(&mut self.frame_composer, dt);
            }
        }
    }
}

impl FlowHandle {
    fn new(tx: Sender<RendererCommand>) -> Self {
        Self { tx }
    }

    pub fn send(&self, command: RendererCommand) {
        if self.tx.send(command).is_err() {
            debug!("Ignoring flow command because receiver dropped");
        }
    }
}
