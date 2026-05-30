use std::sync::RwLock;
use std::{collections::HashMap, rc::Rc, sync::Arc};

use crate::{
    flow::{FrameComposer, SceneFrameInput},
    gpu::{
        buffers::model_matrix::ModelMatrixUniform, glTF::ImportedScene, render_mesh::RenderMesh,
    },
    gui::EguiRenderer,
    renderer::{
        frame::FrameTarget,
        handlers::{InputEvent, asset_handler::AssetHandler},
        renderer_context::RenderContext,
        surface_frame_controller::SurfaceFrameController,
        wrappers::WinitSurfaceProvider,
    },
};
use anyhow::{Result, anyhow};
use bytemuck::bytes_of;
use glam::{Vec3, Vec4};
use hyakou_core::{
    animations::Animator,
    components::{
        AssetType,
        camera::{
            camera::Camera,
        },
        light::LightSource,
    },
    geometry::ray::{Ray, math::intersect_transformed_mesh},
    selection::structure::{SelectionScope, SelectionTarget},
    types::{
        DeltaTime64, ModelMatrixBindingMode, Size,
        ids::MeshId,
    },
};
use log::error;
use shared::SharedAccess;
use wgpu::{
    BindGroup, Color, Device, Operations, Queue, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, TextureFormat,
};
use winit::window::Window;

pub mod actions;
pub mod frame;
mod gpu_resources;
pub mod handlers;
pub mod renderer_context;
pub mod surface_frame_controller;
pub mod wrappers;

use gpu_resources::SceneGpuResources;

pub struct SceneRenderer {
    inner: RwLock<SceneRendererInner>,
}

struct SceneRendererInner {
    ctx: RenderContext,
    gpu_resources: SceneGpuResources,
    animators: HashMap<MeshId, Animator>,
    asset_manager: AssetHandler,
}

impl SceneRenderer {
    pub async fn new(window: Arc<Window>, camera: &Camera) -> Result<Self> {
        let ctx = RenderContext::new(Some(WinitSurfaceProvider {
            window: window.clone(),
        }))
        .await
        .unwrap();

        let asset_handler = AssetHandler::new(
            ctx.device.clone(),
            ctx.queue.clone(),
            ctx.model_binding_mode,
            ctx.model_bind_group_layout.clone(),
            ctx.material_bind_group_layout.clone(),
        );

        let gpu_resources = SceneGpuResources::new(&ctx, camera)?;

        Ok(Self {
            inner: RwLock::new(SceneRendererInner {
                ctx,
                asset_manager: asset_handler,
                gpu_resources,
                animators: HashMap::new(),
            }),
        })
    }

    fn read_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&SceneRendererInner) -> R,
    {
        f(&self.inner.read().expect("SceneRenderer lock poisoned"))
    }

    fn write_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut SceneRendererInner) -> R,
    {
        f(&mut self.inner.write().expect("SceneRenderer lock poisoned"))
    }

    pub fn render_surface_frame(
        &self,
        surface_frame_controller: &mut SurfaceFrameController,
        window: &Window,
        frame_composer: &mut FrameComposer,
        egui_renderer: Option<&mut EguiRenderer>,
        dt: f64,
        camera: &Camera,
        scene_input: SceneFrameInput<'_>,
    ) -> Result<()> {
        self.write_inner(|inner| {
            inner.update_gpu(dt, camera);

            let Some(mut frame) = surface_frame_controller.begin_frame(window, &mut inner.ctx)?
            else {
                return Ok(());
            };

            let mut egui_renderer = egui_renderer;
            {
                let mut target = frame.target();
                inner.render_scene(&mut target, scene_input);
                frame_composer.compose_frame(
                    &mut target,
                    egui_renderer.as_mut().map(|renderer| &mut **renderer),
                );
            }

            let finish_result = surface_frame_controller.finish_frame(&mut inner.ctx, frame);

            if let Some(egui_renderer) = egui_renderer.as_mut() {
                egui_renderer.free_textures_after_submit();
            }

            finish_result
        })
    }

    pub fn resize_surface(
        &self,
        surface_frame_controller: &mut SurfaceFrameController,
        size: Size,
    ) -> Result<()> {
        self.write_inner(|inner| surface_frame_controller.resize(&mut inner.ctx, size))
    }

    pub fn resolve_selection_target(
        &self,
        ray: &Ray,
        scope: SelectionScope,
    ) -> Option<SelectionTarget> {
        self.read_inner(|inner| inner.resolve_selection_target(ray, scope))
    }

    pub fn viewport_size(&self) -> Size {
        self.read_inner(|inner| inner.viewport_size())
    }

    pub fn upload_imported_scene(
        &self,
        id: String,
        asset_type: AssetType,
        imported_scene: ImportedScene,
        display_file_name: &str,
    ) -> Result<Rc<RenderMesh>> {
        self.write_inner(|inner| {
            inner.upload_imported_scene(id, asset_type, imported_scene, display_file_name)
        })
    }

    pub fn set_outline_color(&self, color: Vec4) {
        self.write_inner(|inner| inner.set_outline_color(color));
    }

    pub fn set_outline(&self, color: Vec4, thickness: f32) {
        self.write_inner(|inner| inner.set_outline(color, thickness));
    }

    pub fn get_device(&self) -> Arc<Device> {
        self.read_inner(|inner| inner.ctx.device.clone())
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.read_inner(|inner| {
            inner
                .ctx
                .surface_configuration
                .as_ref()
                .expect("renderer surface must be configured")
                .format
        })
    }
}

impl SceneRendererInner {
    fn update_gpu(&mut self, delta_time: DeltaTime64, camera: &Camera) {
        self.animators.values_mut().for_each(|animator| {
            if let Err(animator_error) = animator.play(delta_time) {
                error!("{:?}", animator_error)
            }
        });

        self.gpu_resources.update(&self.ctx.queue, camera);
    }

    fn resolve_selection_target(
        &self,
        ray: &Ray,
        scope: SelectionScope,
    ) -> Option<SelectionTarget> {
        let hit_mesh_id = self.ray_cast(ray)?;
        let outline_mesh_ids = self.asset_manager.selection_ids_for(&hit_mesh_id, &scope);

        Some(SelectionTarget::new(hit_mesh_id, outline_mesh_ids, scope))
    }

    fn viewport_size(&self) -> Size {
        self.ctx.size
    }

    fn upload_imported_scene(
        &mut self,
        id: String,
        asset_type: AssetType,
        imported_scene: ImportedScene,
        display_file_name: &str,
    ) -> Result<Rc<RenderMesh>> {
        let render_mesh = self
            .asset_manager
            .upload_imported_scene(id, asset_type, imported_scene)
            .ok_or_else(|| {
                anyhow!("uploaded asset `{display_file_name}` produced no renderable meshes")
            })?;

        if asset_type == AssetType::LIGHT {
            self.set_light(LightSource::new(render_mesh.transform.clone(), Vec3::ONE))?;
        }

        Ok(render_mesh)
    }

    fn set_outline_color(&mut self, color: Vec4) {
        self.gpu_resources.set_outline_color(&self.ctx.queue, color);
    }

    fn set_outline(&mut self, color: Vec4, thickness: f32) {
        self.gpu_resources.set_outline(&self.ctx, color, thickness);
    }

    fn set_light(&mut self, light: LightSource) -> Result<()> {
        self.gpu_resources.set_light(&self.ctx, light)
    }

    fn render_scene(&mut self, target: &mut FrameTarget<'_>, input: SceneFrameInput<'_>) {
        {
            target.encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Main Command Buffer"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: target.color_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Color {
                            r: 0.3,
                            g: 0.2,
                            b: 0.8,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                multiview_mask: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: target.depth_view,
                    depth_ops: Some(Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
            });
        }

        if let Some(light_bind_group) = self.gpu_resources.light_bind_group() {
            let camera_bind_group = self.gpu_resources.camera_bind_group();
            let model_binding_mode = self.ctx.model_binding_mode;
            let light_render_pipeline = &self.ctx.light_render_pipeline;
            let no_light_render_pipeline = &self.ctx.no_light_render_pipeline;

            self.asset_manager
                .get_all_visible_assets_with_modifier(&AssetType::NORMAL)
                .for_each(|elem| {
                    Self::record_scene_pass_command_encoder(
                        target,
                        elem,
                        light_render_pipeline,
                        model_binding_mode,
                        camera_bind_group,
                        light_bind_group,
                    );
                });

            self.asset_manager
                .get_all_visible_assets_with_modifier(&AssetType::LIGHT)
                .for_each(|elem| {
                    Self::record_scene_pass_command_encoder(
                        target,
                        elem,
                        no_light_render_pipeline,
                        model_binding_mode,
                        camera_bind_group,
                        light_bind_group,
                    );
                });
        }

        self.render_outlined_meshes(target, input.outlined_mesh_ids);
    }

    fn render_outlined_meshes(
        &mut self,
        target: &mut FrameTarget<'_>,
        outlined_mesh_ids: &[MeshId],
    ) {
        if outlined_mesh_ids.is_empty() {
            return;
        }

        let Some(outline_bind_group) = self.gpu_resources.outline_bind_group() else {
            return;
        };

        let mut render_pass = target.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Outline Command Buffer"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target.color_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            multiview_mask: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: target.depth_view,
                depth_ops: Some(Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&self.ctx.outline_render_pipeline);
        render_pass.set_bind_group(0, self.gpu_resources.camera_bind_group(), &[]);
        render_pass.set_bind_group(
            Self::outline_bind_group_index(self.ctx.model_binding_mode),
            outline_bind_group,
            &[],
        );

        for outlined_mesh_id in outlined_mesh_ids {
            let Some(render_mesh) = self.asset_manager.get_visible_asset(outlined_mesh_id) else {
                continue;
            };

            Self::record_outline_draw_commands(
                &mut render_pass,
                render_mesh,
                target.queue,
                self.ctx.model_binding_mode,
            );
        }
    }

    fn record_scene_pass_command_encoder(
        target: &mut FrameTarget<'_>,
        render_mesh: &RenderMesh,
        render_pipeline: &RenderPipeline,
        model_binding_mode: ModelMatrixBindingMode,
        camera_bind_group: &BindGroup,
        light_bind_group: &BindGroup,
    ) {
        let mut render_pass = target.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Main Command Buffer"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target.color_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            multiview_mask: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: target.depth_view,
                depth_ops: Some(Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(render_pipeline);
        Self::apply_model_matrix(
            &mut render_pass,
            render_mesh,
            target.queue,
            model_binding_mode,
            2,
        );
        render_pass.set_vertex_buffer(0, render_mesh.vertex_buffer.slice(..));
        render_pass.set_bind_group(1, light_bind_group, &[]);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(
            Self::material_bind_group_index(model_binding_mode),
            &render_mesh.material.bind_group,
            &[],
        );
        render_pass.set_index_buffer(
            render_mesh.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..render_mesh.index_count, 0, 0..1);
    }

    fn record_outline_draw_commands(
        render_pass: &mut wgpu::RenderPass<'_>,
        render_mesh: &RenderMesh,
        queue: &Queue,
        model_binding_mode: ModelMatrixBindingMode,
    ) {
        Self::apply_model_matrix(render_pass, render_mesh, queue, model_binding_mode, 1);
        render_pass.set_vertex_buffer(0, render_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            render_mesh.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..render_mesh.index_count, 0, 0..1);
    }

    fn apply_model_matrix(
        render_pass: &mut wgpu::RenderPass<'_>,
        render_mesh: &RenderMesh,
        queue: &Queue,
        model_binding_mode: ModelMatrixBindingMode,
        uniform_bind_group_index: u32,
    ) {
        let model_matrix = render_mesh.transform.read_shared(|t| t.get_matrix());
        match model_binding_mode {
            ModelMatrixBindingMode::Immediate => {
                render_pass.set_immediates(0, bytes_of(&model_matrix));
            }
            ModelMatrixBindingMode::Uniform => {
                let model_uniform = ModelMatrixUniform::new(model_matrix);
                let model_uniform_buffer = render_mesh.model_uniform_buffer.as_ref().expect(
                    "Uniform model binding mode requires a model uniform buffer on RenderMesh",
                );
                let model_bind_group = render_mesh
                    .model_bind_group
                    .as_ref()
                    .expect("Uniform model binding mode requires a model bind group on RenderMesh");
                queue.write_buffer(model_uniform_buffer, 0, bytes_of(&model_uniform));
                render_pass.set_bind_group(uniform_bind_group_index, model_bind_group, &[]);
            }
        }
    }

    fn outline_bind_group_index(model_binding_mode: ModelMatrixBindingMode) -> u32 {
        match model_binding_mode {
            ModelMatrixBindingMode::Immediate => 1,
            ModelMatrixBindingMode::Uniform => 2,
        }
    }

    fn material_bind_group_index(model_binding_mode: ModelMatrixBindingMode) -> u32 {
        match model_binding_mode {
            ModelMatrixBindingMode::Immediate => 2,
            ModelMatrixBindingMode::Uniform => 3,
        }
    }

    fn ray_cast(&self, ray: &Ray) -> Option<MeshId> {
        let mut closest_hit: Option<(MeshId, f32)> = None;

        for render_mesh in self.asset_manager.get_all_visible_assets() {
            let hit = render_mesh
                .transform
                .try_read_shared(|transform| {
                    intersect_transformed_mesh(ray, &render_mesh.mesh, transform)
                })
                .ok()
                .flatten();

            let Some(hit) = hit else {
                continue;
            };

            if closest_hit
                .as_ref()
                .is_none_or(|(_, distance)| hit.distance < *distance)
            {
                closest_hit = Some((render_mesh.id.clone(), hit.distance));
            }
        }

        closest_hit.map(|(mesh_id, _)| mesh_id)
    }
}
