use parking_lot::RwLock;
use std::{path::Path, sync::Arc};

use anyhow::Result;
use bytemuck::bytes_of;
use glam::Vec3;
use log::{error, warn};
use wgpu::{
    BindGroup, Color, CommandEncoder, CommandEncoderDescriptor, Operations,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RenderPipeline, ShaderStages, TextureView, TextureViewDescriptor,
};
use winit::{dpi::PhysicalPosition, window::Window};

use crate::renderer::{
    components::{
        LightType,
        camera::{Camera, CameraUniform},
        light::LightSource,
        render_mesh::RenderMesh,
        transform::Transform,
    },
    geometry::BindGroupProvider,
    handlers::{asset_handler::AssetHandler, camera_controller::CameraController},
    renderer_context::RenderContext,
    trajectory::{Trajectory, linear::LinearTrajectory},
    types::{DeltaTime, TransformBuffer, ids::UniformBufferId, uniform::UniformBuffer},
    wrappers::WinitSurfaceProvider,
};

pub mod components;
pub mod geometry;
pub mod handlers;
pub mod renderer_context;
pub mod trajectory;
pub mod types;
pub mod util;
pub mod wrappers;

pub struct Renderer {
    ctx: RenderContext,
    window: Arc<Window>,
    frame_idx: u8,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_uniform_buffer: UniformBuffer,
    camera_bind_group: BindGroup,
    light: LightSource,
    light_uniform_buffer: UniformBuffer,
    light_bind_group: BindGroup,
    linear_trajectory: LinearTrajectory,
    pub camera_controller: CameraController,
    pub asset_manager: AssetHandler,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        const CAMERA_SPEED_UNITS_PER_SECOND: f32 = 20.0;
        let ctx = RenderContext::new(Some(WinitSurfaceProvider {
            window: window.clone(),
        }))
        .await
        .unwrap();

        let assets_dir = util::get_relative_path();

        let mut asset_handler = AssetHandler::new(ctx.device.clone());
        let _suzanne_mesh = asset_handler.add_from_path(
            "Suzanne".to_string(),
            LightType::LIGHT,
            &Path::new(&assets_dir).join("assets/gltf/Suzanne.gltf"),
        );
        let cube_light_mesh = asset_handler.add_from_path(
            "Cube".to_string(),
            LightType::NO_LIGHT,
            assets_dir.join("assets/gltf/Cube.gltf").as_path(),
        );
        cube_light_mesh
            .as_ref()
            .unwrap()
            .transform
            .write()
            .translate(Vec3::new(0.0, 1.0, 1.0));
        let light = LightSource::new(
            cube_light_mesh.as_ref().unwrap().transform.clone(),
            Vec3::new(1.0, 1.0, 1.0),
        );
        let light_uniform_buffer = UniformBuffer::new(
            UniformBufferId::new("Light Uniform Buffer".to_string()),
            &ctx.device,
            bytes_of(&light.to_gpu().unwrap()),
            cube_light_mesh.as_ref().unwrap().transform.clone(),
        );

        let light_bind_group = LightSource::bind_group(
            &ctx.device,
            &light_uniform_buffer,
            &LightSource::bind_group_layout(&ctx.device),
        );

        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 15.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::Y,
            (ctx.size.width / ctx.size.height) as f32,
            45.0_f32.to_radians(),
            0.1,
            1000.0,
        );

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update(&camera);

        let camera_uniform_buffer = UniformBuffer::new(
            UniformBufferId::new("Camera".to_string()),
            &ctx.device,
            bytemuck::bytes_of(&camera_uniform),
            Arc::new(RwLock::new(Transform::default())),
        );
        let camera_bind_group = CameraUniform::bind_group(
            &ctx.device,
            &camera_uniform_buffer,
            &ctx.camera_bind_group_layout,
        );
        Ok(Self {
            ctx,
            asset_manager: asset_handler,
            frame_idx: 0,
            camera,
            camera_uniform,
            // TODO: Don't hardcode this, however will be resolved in a different ticket
            linear_trajectory: LinearTrajectory::new(
                cube_light_mesh.as_ref().unwrap().as_ref().clone(),
                Vec3::new(0.0, 1.0, 0.0),
                f32::to_radians(0.0),
                f32::to_radians(0.0),
                3.0,
                3.0,
                true,
                true,
            )
            .unwrap(),
            camera_uniform_buffer,
            camera_bind_group,
            light,
            light_uniform_buffer,
            light_bind_group,
            camera_controller: CameraController::new(CAMERA_SPEED_UNITS_PER_SECOND),
            window,
        })
    }

    pub fn update(&mut self, delta_time: DeltaTime) {
        // delta_time is now in seconds (e.g., 0.016 for 60 FPS)
        self.camera_controller
            .update_camera(&mut self.camera, delta_time);
        match self.linear_trajectory.animate(None, delta_time) {
            Ok(_) => {}
            Err(e) => {
                error!(
                    "Failed to animate linear trajectory: {:?} with error: {:?}",
                    self.linear_trajectory.id, e
                )
            }
        }

        self.camera_uniform.update(&self.camera);
        if let Some(gpu_light_source) = self.light.to_gpu() {
            self.light_uniform_buffer
                .update_buffer_transform(&self.ctx.queue, bytes_of(&gpu_light_source))
                .unwrap()
        } else {
            warn!("Skipping light buffer - Transform in Light is still locked");
        }
        self.ctx.queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytes_of(&self.camera_uniform),
        );
    }

    pub fn render(&mut self, mouse_pos: PhysicalPosition<f64>) -> Result<()> {
        self.window.request_redraw();
        if self.ctx.surface_configuration.is_none() {
            return Ok(());
        }

        let output = self.ctx.surface.as_ref().unwrap().get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Rendering Encoder"),
            });

        let mut clear_encoder = self
            .ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Rendering Encoder"),
            });
        let depth_texture = self.ctx.depth_texture.clone();

        {
            clear_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Main Command Buffer"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Color {
                            r: 0.25,
                            g: (0.1),
                            b: (0.75),
                            a: 0.2,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                timestamp_writes: None,
                occlusion_query_set: None,
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &depth_texture.view,
                    depth_ops: Some(Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
            });
        }

        self.asset_manager
            .get_all_visible_assets_with_modifier(&LightType::LIGHT)
            .for_each(|elem| {
                Self::record_scene_pass_command_encoder(
                    &mut encoder,
                    elem,
                    &self.ctx.light_render_pipeline,
                    &self.camera_bind_group,
                    &self.light_bind_group,
                    &view,
                    &depth_texture.view,
                    mouse_pos,
                );
            });

        self.asset_manager
            .get_all_visible_assets_with_modifier(&LightType::NO_LIGHT)
            .for_each(|elem| {
                Self::record_scene_pass_command_encoder(
                    &mut encoder,
                    elem,
                    &self.ctx.no_light_render_pipeline,
                    &self.camera_bind_group,
                    &self.light_bind_group,
                    &view,
                    &depth_texture.view,
                    mouse_pos,
                );
            });

        self.ctx
            .queue
            .submit(std::iter::once(clear_encoder.finish()));
        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.frame_idx = (self.frame_idx + 1) % 1;
        Ok(())
    }

    fn record_scene_pass_command_encoder(
        encoder: &mut CommandEncoder,
        render_mesh: &RenderMesh,
        render_pipeline: &RenderPipeline,
        camera_bind_group: &BindGroup,
        light_bind_group: &BindGroup,
        view: &TextureView,
        depth_view: &TextureView,
        _mouse_pos: PhysicalPosition<f64>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Main Command Buffer"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            timestamp_writes: None,
            occlusion_query_set: None,
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(render_pipeline);
        render_pass.set_push_constants(
            ShaderStages::VERTEX,
            0,
            bytemuck::bytes_of(&render_mesh.transform.read().get_matrix()),
        );
        render_pass.set_vertex_buffer(0, render_mesh.vertex_buffer.slice(..));
        render_pass.set_bind_group(1, light_bind_group, &[]);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_index_buffer(
            render_mesh.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..render_mesh.index_count as u32, 0, 0..1);
    }
}
