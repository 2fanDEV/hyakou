use std::sync::mpsc::{Receiver, channel};

use hyakou_core::Shared;
use log::{debug, warn};

use crate::{
    flow::{
        AssetUploadController, FlowCommandSender, FrameComposer, InputController, RenderController,
        RendererCommand,
    },
    renderer::SceneRenderer,
};

pub struct FlowController {
    rx: Receiver<RendererCommand>,
    render_controller: RenderController,
    frame_composer: FrameComposer,
    input_controller: InputController,
    asset_upload_controller: AssetUploadController,
}

#[derive(Clone)]
pub struct FlowHandle {
    commands: FlowCommandSender,
}

impl FlowController {
    const MAX_COMMANDS_PER_TICK: usize = 128;

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_pair() -> (Self, FlowHandle) {
        let (tx, rx) = channel::<RendererCommand>();
        let commands = FlowCommandSender::new(tx);
        let controller = Self {
            rx,
            render_controller: RenderController::new(commands.clone()),
            frame_composer: FrameComposer::new(),
            input_controller: InputController::new(commands.clone()),
            asset_upload_controller: AssetUploadController::new(commands.clone()),
        };

        (controller, FlowHandle::new(commands))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new_pair(
        upload_status_callback: Shared<Option<js_sys::Function>>,
    ) -> (Self, FlowHandle) {
        let (tx, rx) = channel::<RendererCommand>();
        let commands = FlowCommandSender::new(tx);
        let controller = Self {
            rx,
            render_controller: RenderController::new(commands.clone()),
            frame_composer: FrameComposer::new(),
            input_controller: InputController::new(commands.clone()),
            asset_upload_controller: AssetUploadController::new(
                commands.clone(),
                upload_status_callback,
            ),
        };

        (controller, FlowHandle::new(commands))
    }
    pub fn get_renderer(&self) -> Shared<Option<SceneRenderer>> {
        self.render_controller.renderer()
    }

    pub fn handle_egui_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.render_controller.handle_egui_window_event(event)
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
            RendererCommand::WindowCreated(window) => {
                self.render_controller.handle_window_created(window)
            }
            RendererCommand::AnimateCamera(request) => {
                self.render_controller.animate_camera(request)
            }
            RendererCommand::StopCameraAnimation => self.render_controller.stop_camera_animation(),
            RendererCommand::CursorInWindow { is_inside } => {
                self.input_controller.handle_cursor_in_window(is_inside)
            }
            RendererCommand::CursorMoved { x, y } => {
                self.input_controller.handle_cursor_moved(x, y);
            }
            RendererCommand::KeyboardInput { key, pressed } => {
                let renderer = self.render_controller.renderer();
                self.input_controller
                    .handle_keyboard_input(&renderer, key, pressed);
            }
            RendererCommand::MouseMotion { dx, dy, dt } => {
                let renderer = self.render_controller.renderer();
                self.input_controller
                    .handle_mouse_motion(&renderer, dx, dy, dt);
            }
            RendererCommand::MouseButton { button, pressed } => {
                let renderer = self.render_controller.renderer();
                self.input_controller.handle_mouse_button(
                    &renderer,
                    self.render_controller.window(),
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
                &self.render_controller.renderer(),
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
                .render_controller
                .render_frame(&mut self.frame_composer, dt),
            RendererCommand::Resize { dt, width, height } => {
                self.render_controller.handle_resize(width, height);
                self.render_controller
                    .render_frame(&mut self.frame_composer, dt);
            }
        }
    }
}

impl FlowHandle {
    fn new(commands: FlowCommandSender) -> Self {
        Self { commands }
    }

    pub fn send(&self, command: RendererCommand) {
        if !self.commands.send(command) {
            debug!("Ignoring flow command because receiver dropped");
        }
    }
}
