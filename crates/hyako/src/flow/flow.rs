use std::sync::{
    Arc,
    mpsc::{Receiver, channel},
};

use hyakou_core::{selection::structure::SelectionTarget, types::ids::MeshId};
use log::{debug, warn};
use shared::Shared;

use crate::{
    flow::{
        AssetUploadController, CameraController, FlowCommand, FlowCommandSender, FrameComposer,
        InputController, RenderController, selection_controller::SelectionController,
    },
    renderer::SceneRenderer,
};

pub struct FlowController {
    rx: Receiver<FlowCommand>,
    render_controller: RenderController,
    frame_composer: FrameComposer,
    input_controller: InputController,
    asset_upload_controller: AssetUploadController,
    selection_controller: SelectionController,
}

#[derive(Clone)]
pub struct FlowHandle {
    commands: FlowCommandSender,
}

impl FlowController {
    const MAX_COMMANDS_PER_TICK: usize = 128;

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_pair(
        renderer_view: Shared<Option<Arc<SceneRenderer>>>,
        camera_view: Shared<Option<Arc<CameraController>>>,
    ) -> (Self, FlowHandle) {
        let (tx, rx) = channel::<FlowCommand>();
        let commands = FlowCommandSender::new(tx);
        let controller = Self {
            rx,
            render_controller: RenderController::new(commands.clone(), renderer_view, camera_view),
            frame_composer: FrameComposer::new(),
            input_controller: InputController::new(commands.clone()),
            asset_upload_controller: AssetUploadController::new(commands.clone()),
            selection_controller: SelectionController::new(),
        };

        (controller, FlowHandle::new(commands))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new_pair(
        renderer_view: Shared<Option<Arc<SceneRenderer>>>,
        camera_view: Shared<Option<Arc<CameraController>>>,
        upload_status_callback: Shared<Option<js_sys::Function>>,
    ) -> (Self, FlowHandle) {
        let (tx, rx) = channel::<FlowCommand>();
        let commands = FlowCommandSender::new(tx);
        let controller = Self {
            rx,
            render_controller: RenderController::new(commands.clone(), renderer_view, camera_view),
            frame_composer: FrameComposer::new(),
            input_controller: InputController::new(commands.clone()),
            asset_upload_controller: AssetUploadController::new(
                commands.clone(),
                upload_status_callback,
            ),
            selection_controller: SelectionController::new(),
        };

        (controller, FlowHandle::new(commands))
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

    pub fn current_selection(&self) -> &[SelectionTarget] {
        self.selection_controller.current_selection()
    }

    pub fn outline_selection(&self) -> &[MeshId] {
        self.selection_controller.outlined_mesh_ids()
    }

    fn handle_command(&mut self, command: FlowCommand) {
        match command {
            FlowCommand::WindowCreated(window) => {
                self.render_controller.handle_window_created(window)
            }
            FlowCommand::RendererInitialized {
                renderer,
                camera_controller,
                asset_handler,
            } => self.render_controller.handle_renderer_initialized(
                renderer,
                camera_controller,
                asset_handler,
            ),
            FlowCommand::AnimateCamera(request) => self.render_controller.animate_camera(request),
            FlowCommand::StopCameraAnimation => self.render_controller.stop_camera_animation(),
            FlowCommand::SetCameraMode(mode) => self.render_controller.set_camera_mode(mode),
            FlowCommand::CursorInWindow { is_inside } => {
                self.input_controller.handle_cursor_in_window(is_inside)
            }
            FlowCommand::CursorMoved { x, y } => {
                self.input_controller.handle_cursor_moved(x, y);
            }
            FlowCommand::KeyboardInput { key, pressed } => {
                let events = self.input_controller.handle_keyboard_input(key, pressed);
                self.render_controller.handle_input_events(events);
            }
            FlowCommand::MouseMotion { dx, dy, dt } => {
                let events = self.input_controller.handle_mouse_motion(dx, dy);
                let mouse_delta = self.input_controller.mouse_delta();
                self.render_controller.handle_input_events(events);
                self.render_controller
                    .handle_mouse_movement(&mouse_delta, dt);
            }
            FlowCommand::MouseButton { button, pressed } => {
                let _ = self
                    .input_controller
                    .handle_mouse_button(self.render_controller.window(), button, pressed)
                    .map(|events| self.render_controller.handle_input_events(events));
            }
            FlowCommand::AssetUploadRequested {
                id,
                file_name,
                asset_type,
                bytes,
            } => self
                .asset_upload_controller
                .handle_asset_upload_requested(id, file_name, asset_type, bytes),
            FlowCommand::AssetBundleUploadRequested {
                id,
                file_name,
                asset_type,
                files,
            } => self
                .asset_upload_controller
                .handle_asset_bundle_upload_requested(id, file_name, asset_type, files),
            FlowCommand::ApplyParsedAsset {
                id,
                file_name,
                asset_type,
                imported_scene,
            } => {
                let diagnostics = imported_scene.diagnostics.clone();
                match self.render_controller.apply_parsed_asset(
                    id.clone(),
                    &file_name,
                    asset_type,
                    imported_scene,
                ) {
                    Ok(_) => self.asset_upload_controller.handle_asset_upload_succeeded(
                        id,
                        file_name,
                        diagnostics,
                    ),
                    Err(err) => self.asset_upload_controller.handle_asset_upload_failed(
                        id,
                        file_name,
                        err.to_string(),
                    ),
                }
            }
            FlowCommand::AssetUploadFailed {
                id,
                file_name,
                error,
            } => self
                .asset_upload_controller
                .handle_asset_upload_failed(id, file_name, error),
            FlowCommand::Redraw { dt } => {
                let scene_input = self.selection_controller.scene_frame_input();
                self.render_controller
                    .render_frame(&mut self.frame_composer, dt, scene_input);
            }
            FlowCommand::Resize { dt, width, height } => {
                self.render_controller.handle_resize(width, height);
                let scene_input = self.selection_controller.scene_frame_input();
                self.render_controller
                    .render_frame(&mut self.frame_composer, dt, scene_input);
            }
            FlowCommand::SelectAtScreenPoint { x, y, scope } => self
                .selection_controller
                .select_at_screen_point(&self.render_controller, x, y, scope),
        }
    }
}

impl FlowHandle {
    fn new(commands: FlowCommandSender) -> Self {
        Self { commands }
    }

    pub fn send(&self, command: FlowCommand) {
        if !self.commands.send(command) {
            debug!("Ignoring flow command because receiver dropped");
        }
    }
}
