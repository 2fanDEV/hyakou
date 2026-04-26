use std::sync::Arc;

use hyakou_core::{
    Shared, SharedAccess, components::camera::data_structures::CameraAnimationRequest, shared,
};
use log::{error, warn};
use winit::window::Window;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

use crate::{
    flow::{FlowCommandSender, FrameComposer},
    gui::EguiRenderer,
    renderer::{SceneRenderer, surface_frame_controller::SurfaceFrameController},
};

pub struct RenderController {
    _commands: FlowCommandSender,
    surface_frame_controller: SurfaceFrameController,
    renderer: Shared<Option<SceneRenderer>>,
    egui_renderer: Shared<Option<EguiRenderer>>,
    window: Option<Arc<Window>>,
}

impl RenderController {
    pub fn new(commands: FlowCommandSender) -> Self {
        Self {
            _commands: commands,
            surface_frame_controller: SurfaceFrameController::new(),
            renderer: shared(None),
            egui_renderer: shared(None),
            window: None,
        }
    }

    pub fn renderer(&self) -> Shared<Option<SceneRenderer>> {
        self.renderer.clone()
    }

    pub fn window(&self) -> Option<&Window> {
        self.window.as_deref()
    }

    pub fn handle_egui_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.egui_renderer
            .try_write_shared(|egui_renderer| {
                egui_renderer
                    .as_mut()
                    .is_some_and(|egui_renderer| egui_renderer.handle_window_event(event))
            })
            .unwrap_or(false)
    }

    pub fn handle_window_created(&mut self, window: Arc<Window>) {
        self.window = Some(window.clone());

        let has_renderer = self
            .renderer
            .read_shared(|renderer_slot| renderer_slot.is_some());
        if has_renderer {
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        match pollster::block_on(SceneRenderer::new(window)) {
            Ok(renderer) => {
                let _ = self
                    .renderer
                    .try_write_shared(|renderer_slot| *renderer_slot = Some(renderer));
            }
            Err(renderer_error) => {
                error!("Failed to initialize renderer: {renderer_error:?}");
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        self.create_egui_renderer();

        #[cfg(target_arch = "wasm32")]
        {
            let renderer_slot = self.renderer.clone();
            spawn_local(async move {
                match SceneRenderer::new(window.clone()).await {
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

    pub fn handle_resize(&mut self, width: f64, height: f64) {
        let surface_frame_controller = &mut self.surface_frame_controller;
        if let Err(lock_error) = self.renderer.try_write_shared(|renderer| {
            let Some(renderer) = renderer.as_mut() else {
                return;
            };

            let size = SurfaceFrameController::size_from_dimensions(width, height);
            renderer.set_camera_aspect_from_size(size);
            if let Err(resize_error) =
                surface_frame_controller.resize(renderer.render_context_mut(), size)
            {
                error!("Failed to resize renderer: {resize_error:?}");
            }
        }) {
            error!("Failed to acquire renderer lock during resize: {lock_error:?}");
        }
    }

    pub fn render_frame(&mut self, frame_composer: &mut FrameComposer, dt: f64) {
        let Some(window) = self.window.clone() else {
            return;
        };
        let surface_frame_controller = &mut self.surface_frame_controller;
        let _ = self.renderer.try_write_shared(|renderer_slot| {
            let Some(renderer) = renderer_slot.as_mut() else {
                return;
            };

            let render_result = self.egui_renderer.try_write_shared(|egui_renderer| {
                Self::render_locked_frame(
                    surface_frame_controller,
                    &window,
                    frame_composer,
                    renderer,
                    egui_renderer.as_mut(),
                    dt,
                )
            });

            match render_result {
                Ok(Ok(())) => {}
                Ok(Err(render_error)) => {
                    error!("Renderer frame composition failed: {render_error:?}");
                }
                Err(lock_error) => {
                    warn!(
                        "Rendering frame without egui because renderer slot is busy: {lock_error:?}"
                    );
                    if let Err(render_error) = Self::render_locked_frame(
                        surface_frame_controller,
                        &window,
                        frame_composer,
                        renderer,
                        None,
                        dt,
                    ) {
                        error!("Renderer frame composition failed: {render_error:?}");
                    }
                }
            }
        });
    }

    fn render_locked_frame(
        surface_frame_controller: &mut SurfaceFrameController,
        window: &Window,
        frame_composer: &mut FrameComposer,
        renderer: &mut SceneRenderer,
        mut egui_renderer: Option<&mut EguiRenderer>,
        dt: f64,
    ) -> anyhow::Result<()> {
        renderer.update(dt);

        let Some(mut frame) =
            surface_frame_controller.begin_frame(window, renderer.render_context_mut())?
        else {
            return Ok(());
        };

        {
            let mut target = frame.target();
            frame_composer.compose_frame(
                &mut target,
                renderer,
                egui_renderer.as_mut().map(|renderer| &mut **renderer),
            );
        }

        let finish_result =
            surface_frame_controller.finish_frame(renderer.render_context_mut(), frame);

        if let Some(egui_renderer) = egui_renderer.as_mut() {
            egui_renderer.free_textures_after_submit();
        }

        finish_result
    }

    pub fn animate_camera(&mut self, request: CameraAnimationRequest) {
        let _ = self.renderer.try_write_shared(|renderer_slot| {
            let Some(renderer) = renderer_slot.as_mut() else {
                return;
            };
            renderer
                .camera_handler
                .state
                .animate_camera(&renderer.camera, request);
        });
    }

    pub fn stop_camera_animation(&mut self) {
        let _ = self.renderer.try_write_shared(|renderer_slot| {
            let Some(renderer) = renderer_slot.as_mut() else {
                return;
            };

            renderer
                .camera_handler
                .state
                .stop_camera_animation(&renderer.camera.id);
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn create_egui_renderer(&mut self) {
        let egui_renderer = self
            .renderer
            .try_read_shared(|renderer| {
                if let Some(renderer) = renderer {
                    use egui_wgpu::RendererOptions;
                    Some(EguiRenderer::new(
                        renderer.get_device().clone(),
                        self.window.as_ref().unwrap().clone(),
                        renderer.get_surface_configuration().format,
                        RendererOptions::default(),
                    ))
                } else {
                    error!("Renderer is not initialized yet!");
                    None
                }
            })
            .unwrap();
        self.egui_renderer
            .try_write_shared(|slot| *slot = egui_renderer)
            .unwrap();
    }
}
