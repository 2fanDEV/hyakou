use std::{collections::HashMap, f32::consts::PI, sync::Arc};

use crate::{
    flow::SceneFrameInput,
    gpu::{buffers::model_matrix::ModelMatrixUniform, render_mesh::RenderMesh},
    renderer::{
        frame::FrameTarget,
        handlers::{asset_handler::AssetHandler, camera::CameraHandler},
        renderer_context::RenderContext,
        wrappers::WinitSurfaceProvider,
    },
};
use anyhow::Result;
use bytemuck::bytes_of;
use glam::{Vec3, Vec4};
use hyakou_core::{
    animations::Animator,
    components::{
        AssetType,
        camera::{camera::Camera, data_structures::CameraMode},
    },
    geometry::ray::{Ray, math::intersect_transformed_mesh},
    selection::structure::{SelectionScope, SelectionTarget},
    types::{
        DeltaTime64, ModelMatrixBindingMode, Size,
        camera::{Pitch, Yaw},
        ids::MeshId,
    },
};
use log::error;
use shared::SharedAccess;
use wgpu::{
    BindGroup, Color, Device, Operations, Queue, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, SurfaceConfiguration,
};
use winit::window::Window;

pub mod actions;
pub mod frame;
pub mod handlers;
pub mod renderer_context;
mod scene_gpu_resources;
pub mod surface_frame_controller;
pub mod util;
pub mod wrappers;

use scene_gpu_resources::SceneGpuResources;

pub struct SceneRenderer {
    pub ctx: RenderContext,
    pub camera: Camera,
    gpu_resources: SceneGpuResources,
    animators: HashMap<MeshId, Animator>,
    pub camera_handler: CameraHandler,
    pub asset_manager: AssetHandler,
}

impl SceneRenderer {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        const CAMERA_SPEED_UNITS_PER_SECOND: f32 = 20.0;
        const CAMERA_SENSITIVITY: f32 = 0.001;
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

        let aspect = Camera::aspect_ratio_from_size(ctx.size);
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 15.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::Y,
            aspect,
            45.0_f32.to_radians(),
            0.1,
            1000.0,
            Yaw::new(-PI / 2.0),
            Pitch::new(0.0),
            CAMERA_SPEED_UNITS_PER_SECOND,
            CAMERA_SENSITIVITY,
            0.5,
        );
        let gpu_resources = SceneGpuResources::new(&ctx, &camera)?;

        Ok(Self {
            ctx,
            asset_manager: asset_handler,
            camera,
            gpu_resources,
            animators: HashMap::new(),
            camera_handler: CameraHandler::new(CameraMode::ORBIT),
        })
    }

    pub fn update(&mut self, delta_time: DeltaTime64) {
        self.camera_handler
            .update(&mut self.camera, delta_time as f32);
        self.animators.values_mut().for_each(|animator| {
            if let Err(animator_error) = animator.play(delta_time) {
                error!("{:?}", animator_error)
            }
        });

        self.gpu_resources.update(&self.ctx.queue, &self.camera);
    }

    pub fn resolve_selection_target(
        &self,
        ray: &Ray,
        scope: SelectionScope,
    ) -> Option<SelectionTarget> {
        let hit_mesh_id = self.ray_cast(ray)?;
        let outline_mesh_ids = self.asset_manager.selection_ids_for(&hit_mesh_id, &scope);

        Some(SelectionTarget::new(hit_mesh_id, outline_mesh_ids, scope))
    }

    pub fn set_outline_color(&mut self, color: Vec4) {
        self.gpu_resources.set_outline_color(&self.ctx.queue, color);
    }

    pub fn render_scene(&mut self, target: &mut FrameTarget<'_>, input: SceneFrameInput<'_>) {
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

        self.asset_manager
            .get_all_visible_assets_with_modifier(&AssetType::NORMAL)
            .for_each(|elem| {
                Self::record_scene_pass_command_encoder(
                    target,
                    elem,
                    &self.ctx.light_render_pipeline,
                    self.ctx.model_binding_mode,
                    &self.gpu_resources.camera_bind_group,
                    &self.gpu_resources.light_bind_group,
                );
            });

        self.asset_manager
            .get_all_visible_assets_with_modifier(&AssetType::LIGHT)
            .for_each(|elem| {
                Self::record_scene_pass_command_encoder(
                    target,
                    elem,
                    &self.ctx.no_light_render_pipeline,
                    self.ctx.model_binding_mode,
                    &self.gpu_resources.camera_bind_group,
                    &self.gpu_resources.light_bind_group,
                );
            });

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
        render_pass.set_bind_group(0, &self.gpu_resources.camera_bind_group, &[]);
        render_pass.set_bind_group(
            Self::outline_bind_group_index(self.ctx.model_binding_mode),
            &self.gpu_resources.outline_bind_group,
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

    pub fn material_bind_group_index(model_binding_mode: ModelMatrixBindingMode) -> u32 {
        match model_binding_mode {
            ModelMatrixBindingMode::Immediate => 2,
            ModelMatrixBindingMode::Uniform => 3,
        }
    }

    pub fn get_device(&self) -> Arc<Device> {
        self.ctx.device.clone()
    }

    pub fn get_queue(&self) -> &Queue {
        &self.ctx.queue
    }

    pub fn get_surface_configuration(&self) -> &SurfaceConfiguration {
        self.ctx.surface_configuration.as_ref().unwrap()
    }

    pub(crate) fn render_context_mut(&mut self) -> &mut RenderContext {
        &mut self.ctx
    }

    pub(crate) fn set_camera_aspect_from_size(&mut self, size: Size) {
        if !size.is_zero() {
            self.camera.set_aspect_from_size(size);
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
