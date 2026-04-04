use std::sync::{
    Arc,
    mpsc::{Receiver, Sender, channel},
};

use hyakou_core::{
    Shared, SharedAccess,
    components::{LightType, mesh_node::MeshNode},
    shared,
    types::mouse_delta::{MouseAction, MouseDelta, MousePosition, MouseState},
};
use log::{debug, error, warn};
use winit::window::{CursorGrabMode, Window};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

use crate::{
    flow::RendererCommand,
    renderer::{
        Renderer,
        handlers::{InputEvent, keyboard_handler::KeyboardHandler, mouse_handler::MouseHandler},
        util,
    },
};

pub struct FlowController {
    tx: Sender<RendererCommand>,
    rx: Receiver<RendererCommand>,
    renderer: Shared<Option<Renderer>>,
    keyboard_handler: KeyboardHandler,
    mouse_handler: MouseHandler,
    mouse_delta: MouseDelta,
    window: Option<Arc<Window>>,
}

#[derive(Clone)]
pub struct FlowHandle {
    tx: Sender<RendererCommand>,
}

impl FlowController {
    const MAX_COMMANDS_PER_TICK: usize = 128;

    pub fn new_pair() -> (Self, FlowHandle) {
        let (tx, rx) = channel::<RendererCommand>();
        let controller = Self {
            tx: tx.clone(),
            rx,
            renderer: shared(None),
            keyboard_handler: KeyboardHandler::new(),
            mouse_handler: MouseHandler::new(),
            mouse_delta: MouseDelta::default(),
            window: None,
        };

        (controller, FlowHandle::new(tx))
    }

    pub fn get_renderer(&self) -> Shared<Option<Renderer>> {
        self.renderer.clone()
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
            RendererCommand::WindowCreated(window) => self.handle_window_created(window),
            RendererCommand::SetCoords(coordinates) => self.handle_set_coords(coordinates),
            RendererCommand::CursorInWindow { is_inside } => {
                self.mouse_delta.set_is_mouse_on_window(is_inside)
            }
            RendererCommand::CursorMoved { x, y } => {
                self.mouse_delta.position = MousePosition::new(x, y);
            }
            RendererCommand::KeyboardInput { key, pressed } => {
                self.handle_keyboard_input(key, pressed);
            }
            RendererCommand::MouseMotion { dx, dy, dt } => {
                self.handle_mouse_motion(dx, dy, dt);
            }
            RendererCommand::MouseButton { button, pressed } => {
                self.handle_mouse_button(button, pressed);
            }
            RendererCommand::AssetUploadRequested {
                id,
                asset_type,
                bytes,
            } => self.handle_asset_upload_requested(id, asset_type, bytes),
            RendererCommand::ApplyParsedAsset {
                id,
                asset_type,
                mesh_nodes,
            } => self.handle_apply_parsed_asset(id, asset_type, mesh_nodes),
            RendererCommand::AssetUploadFailed { id, error } => {
                error!("Asset upload failed for `{id}`: {error}");
            }
            RendererCommand::Redraw { dt } => self.handle_redraw(dt),
            RendererCommand::Resize { dt, height, width } => {
                self.handle_resize(height, width);
                self.handle_redraw(dt);
            }
        }
    }

    fn handle_window_created(&mut self, window: Arc<Window>) {
        self.window = Some(window.clone());

        let has_renderer = self
            .renderer
            .read_shared(|renderer_slot| renderer_slot.is_some());
        if has_renderer {
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        match pollster::block_on(Renderer::new(window)) {
            Ok(renderer) => {
                let _ = self
                    .renderer
                    .try_write_shared(|renderer_slot| *renderer_slot = Some(renderer));
            }
            Err(renderer_error) => {
                error!("Failed to initialize renderer: {renderer_error:?}");
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let renderer_slot = self.renderer.clone();
            spawn_local(async move {
                match Renderer::new(window.clone()).await {
                    Ok(renderer) => {
                        let Some(()) = renderer_slot
                            .try_write_shared(|slot| *slot = Some(renderer))
                            .ok()
                        else {
                            warn!(
                                "Renderer initialized but flow slot was busy; skipping this frame"
                            );
                            return;
                        };
                        window.request_redraw();
                    }
                    Err(renderer_error) => {
                        error!("Failed to initialize renderer for wasm: {renderer_error:?}");
                    }
                }
            });
        }
    }

    fn handle_set_coords(&mut self, coordinates: hyakou_core::types::shared::Coordinates3) {
        let _ = self.renderer.try_write_shared(|renderer_slot| {
            let Some(renderer) = renderer_slot.as_mut() else {
                return;
            };
            renderer
                .camera_handler
                .state
                .set_camera_transition(&mut renderer.camera, coordinates);
        });
    }

    fn handle_keyboard_input(&mut self, key: winit::keyboard::KeyCode, pressed: bool) {
        let events = self.keyboard_handler.handle_key(key, pressed);
        let _ = self.renderer.try_write_shared(|renderer_slot| {
            let Some(renderer) = renderer_slot.as_mut() else {
                return;
            };

            for input_event in events {
                Self::handle_input_event(renderer, input_event);
            }
        });
    }

    fn handle_mouse_motion(&mut self, dx: f64, dy: f64, dt: f32) {
        self.mouse_delta.delta_position =
            hyakou_core::types::mouse_delta::MovementDelta::new(dx, dy);

        let _ = self.renderer.try_write_shared(|renderer_slot| {
            let Some(renderer) = renderer_slot.as_mut() else {
                return;
            };

            renderer
                .camera_handler
                .mouse_movement(&mut renderer.camera, &self.mouse_delta, dt);
        });
    }

    fn handle_mouse_button(
        &mut self,
        button: hyakou_core::types::mouse_delta::MouseButton,
        pressed: bool,
    ) {
        self.mouse_delta.state = MouseState::new(
            button,
            if pressed {
                MouseAction::Clicked
            } else {
                MouseAction::Released
            },
        );

        if let Some(window) = self.window.as_ref() {
            let grab_mode = if pressed {
                CursorGrabMode::Locked
            } else {
                CursorGrabMode::None
            };

            if let Err(cursor_error) = window.set_cursor_grab(grab_mode) {
                error!("Failed to set cursor grab mode: {cursor_error:?}");
            }

            window.set_cursor_visible(!pressed);
        }

        let events = self.mouse_handler.handle_button(button, pressed);
        let _ = self.renderer.try_write_shared(|renderer_slot| {
            let Some(renderer) = renderer_slot.as_mut() else {
                return;
            };

            for input_event in events {
                Self::handle_input_event(renderer, input_event);
            }
        });
    }

    fn handle_asset_upload_requested(&mut self, id: String, asset_type: LightType, bytes: Vec<u8>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::gpu::glTF::GLTFLoader;
            let gltf_loader = GLTFLoader::new(util::get_relative_path());
            let parsed_mesh_nodes = pollster::block_on(gltf_loader.load_from_bytes(bytes));
            match parsed_mesh_nodes {
                Ok(mesh_nodes) => {
                    self.send_internal(RendererCommand::ApplyParsedAsset {
                        id,
                        asset_type,
                        mesh_nodes,
                    });
                }
                Err(upload_error) => {
                    self.send_internal(RendererCommand::AssetUploadFailed {
                        id,
                        error: upload_error.to_string(),
                    });
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let tx = self.tx.clone();
            spawn_local(async move {
                use crate::gpu::glTF::GLTFLoader;
                let gltf_loader = GLTFLoader::new(util::get_relative_path());
                let parsed_mesh_nodes = gltf_loader.load_from_bytes(bytes).await;
                let next_command = match parsed_mesh_nodes {
                    Ok(mesh_nodes) => RendererCommand::ApplyParsedAsset {
                        id,
                        asset_type,
                        mesh_nodes,
                    },
                    Err(upload_error) => RendererCommand::AssetUploadFailed {
                        id,
                        error: upload_error.to_string(),
                    },
                };

                if tx.send(next_command).is_err() {
                    warn!("Failed to send parsed asset command: flow channel closed");
                }
            });
        }
    }

    fn handle_apply_parsed_asset(
        &mut self,
        id: String,
        asset_type: LightType,
        mesh_nodes: Vec<MeshNode>,
    ) {
        let _ = self.renderer.try_write_shared(|renderer_slot| {
            let Some(renderer) = renderer_slot.as_mut() else {
                warn!("Dropping parsed asset `{id}` because renderer is not ready");
                return;
            };

            renderer
                .asset_manager
                .upload_mesh_nodes(id, asset_type, mesh_nodes);
        });
    }

    fn handle_redraw(&mut self, dt: f64) {
        let _ = self.renderer.try_write_shared(|renderer_slot| {
            let Some(renderer) = renderer_slot.as_mut() else {
                return;
            };

            renderer.update(dt);
            if let Err(render_error) = renderer.render() {
                error!("Renderer draw call failed: {render_error:?}");
            }
        });
    }

    fn handle_resize(&mut self, height: f64, width: f64) {
        self.renderer
            .try_write_shared(|renderer| {
                let Some(renderer) = renderer.as_mut() else {
                    return;
                };

                renderer.resize(height, width);
            })
            .unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn send_internal(&self, command: RendererCommand) {
        if self.tx.send(command).is_err() {
            warn!("Failed to enqueue flow command: receiver dropped");
        }
    }

    fn handle_input_event(renderer: &mut Renderer, event: InputEvent) {
        match event {
            InputEvent::ActionStarted(action) => {
                renderer.camera_handler.handle_action(&action, true);
            }
            InputEvent::ActionEnded(action) => {
                renderer.camera_handler.handle_action(&action, false);
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
