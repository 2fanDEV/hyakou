use std::sync::Arc;

use anyhow::{Result, anyhow};
use glam::Vec3;
use hyakou_core::{
    components::{
        AssetType,
        camera::{camera::Camera, data_structures::CameraAnimationRequest},
        light::LightSource,
    },
    geometry::ray::Ray,
    selection::structure::{SelectionScope, SelectionTarget},
    types::{Size, mouse_delta::MouseDelta},
};
use log::error;
use shared::{Shared, SharedAccess};
use winit::window::Window;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

use crate::{
    flow::{
        AssetController, CameraController, FlowCommandSender, FrameComposer, SceneFrameInput,
        selection_controller::SelectionContext,
    },
    gpu::{glTF::ImportedScene, render_mesh::RenderMesh},
    gui::EguiRenderer,
    renderer::{
        SceneRenderInput, SceneRenderer, handlers::InputEvent, renderer_context::RenderContext,
        surface_frame_controller::SurfaceFrameController, wrappers::WinitSurfaceProvider,
    },
};

use std::rc::Rc;

pub struct RenderController {
    _commands: FlowCommandSender,
    surface_frame_controller: SurfaceFrameController,
    renderer: Option<Arc<SceneRenderer>>,
    camera_controller: Option<Arc<CameraController>>,
    asset_controller: Option<AssetController>,
    egui_renderer: Option<EguiRenderer>,
    renderer_view: Shared<Option<Arc<SceneRenderer>>>,
    camera_view: Shared<Option<Arc<CameraController>>>,
    window: Option<Arc<Window>>,
}

impl RenderController {
    pub fn new(
        commands: FlowCommandSender,
        renderer_view: Shared<Option<Arc<SceneRenderer>>>,
        camera_view: Shared<Option<Arc<CameraController>>>,
    ) -> Self {
        Self {
            _commands: commands,
            surface_frame_controller: SurfaceFrameController::new(),
            renderer: None,
            camera_controller: None,
            asset_controller: None,
            egui_renderer: None,
            renderer_view,
            camera_view,
            window: None,
        }
    }

    pub fn window(&self) -> Option<&Window> {
        self.window.as_deref()
    }

    pub fn handle_egui_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.egui_renderer
            .as_mut()
            .is_some_and(|egui_renderer| egui_renderer.handle_window_event(event))
    }

    pub fn handle_window_created(&mut self, window: Arc<Window>) {
        self.window = Some(window.clone());

        if self.renderer.is_some() {
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let camera_controller =
                CameraController::new(SurfaceFrameController::size_from_dimensions(1920.0, 1080.0));
            let camera = camera_controller.active_camera();

            match pollster::block_on(initialize_scene_renderer(window, &camera)) {
                Ok((renderer, asset_controller)) => {
                    self.handle_renderer_initialized(renderer, camera_controller, asset_controller);
                }
                Err(renderer_error) => {
                    error!("Failed to initialize renderer: {renderer_error:?}");
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        self.create_egui_renderer();

        #[cfg(target_arch = "wasm32")]
        {
            use crate::flow::FlowCommand;

            let commands = self._commands.clone();
            spawn_local(async move {
                // TODO: get actual viewport size from canvas
                let camera_controller = CameraController::new(
                    SurfaceFrameController::size_from_dimensions(1920.0, 1080.0),
                );
                let camera = camera_controller.active_camera();

                match initialize_scene_renderer(window.clone(), &camera).await {
                    Ok((renderer, asset_controller)) => {
                        commands.send(FlowCommand::RendererInitialized {
                            renderer,
                            camera_controller,
                            asset_controller,
                        });
                        window.request_redraw();
                    }
                    Err(renderer_error) => {
                        error!("Failed to initialize renderer for wasm: {renderer_error:?}");
                    }
                }
            });
        }
    }

    pub fn handle_renderer_initialized(
        &mut self,
        renderer: SceneRenderer,
        camera_controller: CameraController,
        asset_controller: AssetController,
    ) {
        let renderer = Arc::new(renderer);
        let camera_controller = Arc::new(camera_controller);

        self.renderer_view
            .write_shared(|slot| *slot = Some(renderer.clone()));
        self.camera_view
            .write_shared(|slot| *slot = Some(camera_controller.clone()));

        self.renderer = Some(renderer);
        self.camera_controller = Some(camera_controller);
        self.asset_controller = Some(asset_controller);
    }

    pub fn handle_resize(&mut self, width: f64, height: f64) {
        let surface_frame_controller = &mut self.surface_frame_controller;
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        let size = SurfaceFrameController::size_from_dimensions(width, height);
        if let Some(camera_controller) = self.camera_controller.as_mut() {
            camera_controller.set_aspect_from_size(size);
        }
        if let Err(resize_error) = renderer.resize_surface(surface_frame_controller, size) {
            error!("Failed to resize renderer: {resize_error:?}");
        }
    }

    pub fn render_frame(
        &mut self,
        frame_composer: &mut FrameComposer,
        dt: f64,
        scene_input: SceneFrameInput<'_>,
    ) {
        let Some(window) = self.window.clone() else {
            return;
        };
        let surface_frame_controller = &mut self.surface_frame_controller;
        let Some(camera_controller) = self.camera_controller.as_mut() else {
            return;
        };
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        let Some(asset_controller) = self.asset_controller.as_mut() else {
            return;
        };

        camera_controller.update(dt);
        let camera = camera_controller.active_camera();

        let normal_meshes: Vec<Rc<RenderMesh>> = asset_controller
            .visible_meshes_with_asset_type(&AssetType::NORMAL)
            .cloned()
            .collect();
        let light_meshes: Vec<Rc<RenderMesh>> = asset_controller
            .visible_meshes_with_asset_type(&AssetType::LIGHT)
            .cloned()
            .collect();
        let outlined_meshes: Vec<Rc<RenderMesh>> = scene_input
            .outlined_mesh_ids
            .iter()
            .filter_map(|id| asset_controller.get_visible_asset(id).cloned())
            .collect();

        let render_input = SceneRenderInput {
            normal_meshes: &normal_meshes,
            light_meshes: &light_meshes,
            outlined_meshes: &outlined_meshes,
        };

        if let Err(render_error) = renderer.render_surface_frame(
            surface_frame_controller,
            &window,
            frame_composer,
            self.egui_renderer.as_mut(),
            dt,
            &camera,
            render_input,
        ) {
            error!("Renderer frame composition failed: {render_error:?}");
        }
    }

    pub fn animate_camera(&mut self, request: CameraAnimationRequest) {
        if let Some(camera_controller) = self.camera_controller.as_mut() {
            camera_controller.animate_camera(request);
        }
    }

    pub fn stop_camera_animation(&mut self) {
        if let Some(camera_controller) = self.camera_controller.as_mut() {
            camera_controller.stop_camera_animation();
        }
    }

    pub fn set_camera_mode(
        &mut self,
        mode: hyakou_core::components::camera::data_structures::CameraMode,
    ) {
        if let Some(camera_controller) = self.camera_controller.as_mut() {
            camera_controller.set_camera_mode(mode);
        }
    }

    pub fn handle_input_events(&mut self, events: impl IntoIterator<Item = InputEvent>) {
        if let Some(camera_controller) = self.camera_controller.as_mut() {
            camera_controller.handle_input_events(events);
        }
    }

    pub fn handle_mouse_movement(&mut self, mouse_delta: &MouseDelta, dt: f32) {
        if let Some(camera_controller) = self.camera_controller.as_mut() {
            camera_controller.handle_mouse_movement(mouse_delta, dt);
        }
    }

    pub fn apply_parsed_asset(
        &mut self,
        id: String,
        file_name: &str,
        asset_type: AssetType,
        imported_scene: ImportedScene,
    ) -> anyhow::Result<Rc<RenderMesh>> {
        let Some(asset_controller) = self.asset_controller.as_mut() else {
            return Err(anyhow!("asset controller is not ready"));
        };

        let render_mesh = asset_controller
            .upload_imported_scene(id, asset_type, imported_scene)
            .ok_or_else(|| anyhow!("uploaded asset `{file_name}` produced no renderable meshes"))?;

        if asset_type == AssetType::LIGHT {
            if let Some(renderer) = self.renderer.as_ref() {
                renderer.set_light(LightSource::new(render_mesh.transform.clone(), Vec3::ONE))?;
            }
        }

        Ok(render_mesh)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn create_egui_renderer(&mut self) {
        let Some(renderer) = self.renderer.as_ref() else {
            error!("Renderer is not initialized yet!");
            return;
        };

        use egui_wgpu::RendererOptions;
        self.egui_renderer = Some(EguiRenderer::new(
            renderer.get_device().clone(),
            self.window.as_ref().unwrap().clone(),
            renderer.surface_format(),
            RendererOptions::default(),
        ));
    }
}

async fn initialize_scene_renderer(
    window: Arc<Window>,
    camera: &Camera,
) -> Result<(SceneRenderer, AssetController)> {
    let ctx = RenderContext::new(Some(WinitSurfaceProvider { window })).await?;
    let asset_controller = AssetController::from_render_context(&ctx);
    let renderer = SceneRenderer::from_context(ctx, camera)?;

    Ok((renderer, asset_controller))
}

impl SelectionContext for RetnderController {
    fn active_camera(&self) -> Result<hyakou_core::components::camera::camera::Camera> {
        self.camera_controller
            .as_ref()
            .map(|cc| cc.active_camera())
            .ok_or_else(|| anyhow!("Camera controller missing or not initialized"))
    }

    fn viewport_size(&self) -> Result<Size> {
        self.renderer
            .as_ref()
            .map(|renderer| renderer.viewport_size())
            .ok_or_else(|| anyhow!("Renderer missing or not initialized"))
    }

    fn resolve_selection_target(&self, ray: Ray, scope: SelectionScope) -> Option<SelectionTarget> {
        self.asset_controller
            .as_ref()
            .and_then(|ac| ac.resolve_selection_target(&ray, scope))
    }
}
