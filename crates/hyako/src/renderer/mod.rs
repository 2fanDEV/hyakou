use std::{collections::HashMap, f32::consts::PI, sync::Arc};

use crate::{
    gpu::{
        buffers::{
            camera_buffer::CameraUniform, model_matrix::ModelMatrixUniform, uniform::UniformBuffer,
        },
        render_mesh::RenderMesh,
    },
    renderer::{
        handlers::{asset_handler::AssetHandler, camera::CameraHandler},
        renderer_context::RenderContext,
        wrappers::WinitSurfaceProvider,
    },
};
use anyhow::{Result, anyhow};
use bytemuck::bytes_of;
use glam::Vec3;
use hyakou_core::{
    SharedAccess,
    animations::{Animation, Animator, NEUTRAL_SPEED, trajectory::linear::LinearTrajectory},
    components::{
        LightType,
        camera::{camera::Camera, data_structures::CameraMode},
        light::LightSource,
    },
    shared,
    traits::BindGroupProvider,
    types::{
        DeltaTime64, ModelMatrixBindingMode, Size, TransformBuffer,
        camera::{Pitch, Yaw},
        ids::{MeshId, UniformBufferId},
        transform::Transform,
    },
};
use log::{error, warn};
use wgpu::{
    BindGroup, Color, CommandEncoder, CommandEncoderDescriptor, Device, Operations, Queue,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RenderPipeline, SurfaceConfiguration, TextureView, TextureViewDescriptor,
};
use winit::window::Window;

pub mod actions;
pub mod handlers;
pub mod renderer_context;
pub mod util;
pub mod wrappers;

pub struct Renderer {
    ctx: RenderContext,
    window: Arc<Window>,
    frame_idx: u8,
    pub camera: Camera,
    camera_uniform: CameraUniform,
    camera_uniform_buffer: UniformBuffer,
    camera_bind_group: BindGroup,
    light: LightSource,
    light_uniform_buffer: UniformBuffer,
    light_bind_group: BindGroup,
    animators: HashMap<MeshId, Animator>,
    pub camera_handler: CameraHandler,
    pub asset_manager: AssetHandler,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        const CAMERA_SPEED_UNITS_PER_SECOND: f32 = 20.0;
        const CAMERA_SENSITIVITY: f32 = 0.001;
        let ctx = RenderContext::new(Some(WinitSurfaceProvider {
            window: window.clone(),
        }))
        .await
        .unwrap();

        let assets_dir = util::get_relative_path();

        let mut asset_handler = AssetHandler::new(
            ctx.device.clone(),
            ctx.queue.clone(),
            ctx.model_binding_mode,
            ctx.model_bind_group_layout.clone(),
            ctx.material_bind_group_layout.clone(),
        );
        let _suzanne_mesh = asset_handler
            .add_from_path(
                "Suzanne".to_string(),
                LightType::LIGHT,
                assets_dir.join("assets/gltf/Suzanne.gltf").as_path(),
            )
            .await?;
        let cube_light_mesh = asset_handler
            .add_from_path(
                "Cube".to_string(),
                LightType::NO_LIGHT,
                assets_dir.join("assets/gltf/Cube.gltf").as_path(),
            )
            .await?;
        cube_light_mesh
            .transform
            .try_write_shared(|t| t.translate(Vec3::new(0.0, 1.0, 1.0)))?;
        let light = LightSource::new(cube_light_mesh.transform.clone(), Vec3::new(1.0, 1.0, 1.0));
        let light_uniform_buffer = UniformBuffer::new(
            UniformBufferId::new("Light Uniform Buffer".to_string()),
            &ctx.device,
            bytes_of(&light.to_gpu().unwrap()),
            cube_light_mesh.transform.clone(),
        );

        let light_bind_group = LightSource::bind_group(
            &ctx.device,
            &light_uniform_buffer,
            &LightSource::bind_group_layout(&ctx.device),
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

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update(&camera);

        let camera_uniform_buffer = UniformBuffer::new(
            UniformBufferId::new("Camera".to_string()),
            &ctx.device,
            bytemuck::bytes_of(&camera_uniform),
            shared(Transform::default()),
        );
        let camera_bind_group = CameraUniform::bind_group(
            &ctx.device,
            &camera_uniform_buffer,
            &ctx.camera_bind_group_layout,
        );

        let test_trajectory = LinearTrajectory::new_deconstructed_mesh(
            cube_light_mesh.id.clone(),
            cube_light_mesh.transform.clone(),
            Vec3::new(0.0, 1.0, 0.0),
            f32::to_radians(0.0),
            f32::to_radians(0.0),
            3.0,
            3.0,
            true,
            true,
        )
        .unwrap();

        let mut animators = HashMap::<MeshId, Animator>::new();
        animators.insert(
            test_trajectory.get_id().clone(),
            Animator::new(NEUTRAL_SPEED, Box::new(test_trajectory)).unwrap(),
        );

        Ok(Self {
            ctx,
            asset_manager: asset_handler,
            frame_idx: 0,
            camera_uniform,
            camera,
            camera_uniform_buffer,
            camera_bind_group,
            light,
            light_uniform_buffer,
            light_bind_group,
            animators,
            camera_handler: CameraHandler::new(CameraMode::ORBIT),
            window,
        })
    }

    pub fn resize(&mut self, width: f64, height: f64) -> Result<()> {
        let size = Self::size_from_dimensions(width, height);

        if !size.is_zero() {
            self.camera.set_aspect_from_size(size);
        }

        self.ctx.resize(size)
    }

    pub fn update(&mut self, delta_time: DeltaTime64) {
        self.camera_handler
            .update(&mut self.camera, delta_time as f32);
        self.animators.values_mut().for_each(|animator| {
            if let Err(animator_error) = animator.play(delta_time) {
                error!("{:?}", animator_error)
            }
        });

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

    pub fn render(&mut self) -> Result<()> {
        self.window.request_redraw();
        if self.ctx.surface_configuration.is_none() || self.ctx.size.is_zero() {
            return Ok(());
        }

        let (output, should_reconfigure_surface) =
            match self.ctx.surface.as_ref().unwrap().get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(output) => (output, false),
                wgpu::CurrentSurfaceTexture::Suboptimal(output) => (output, true),
                surface_status => return self.handle_surface_acquisition_status(surface_status),
            };

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
                    &self.ctx.queue,
                    self.ctx.model_binding_mode,
                    &self.camera_bind_group,
                    &self.light_bind_group,
                    &view,
                    &depth_texture.view,
                );
            });

        self.asset_manager
            .get_all_visible_assets_with_modifier(&LightType::NO_LIGHT)
            .for_each(|elem| {
                Self::record_scene_pass_command_encoder(
                    &mut encoder,
                    elem,
                    &self.ctx.no_light_render_pipeline,
                    &self.ctx.queue,
                    self.ctx.model_binding_mode,
                    &self.camera_bind_group,
                    &self.light_bind_group,
                    &view,
                    &depth_texture.view,
                );
            });

        self.ctx
            .queue
            .submit(std::iter::once(clear_encoder.finish()));
        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        if should_reconfigure_surface {
            self.ctx.resize(self.ctx.size)?;
        }
        self.frame_idx = (self.frame_idx + 1) % 1;
        Ok(())
    }

    fn handle_surface_acquisition_status(
        &mut self,
        surface_status: wgpu::CurrentSurfaceTexture,
    ) -> Result<()> {
        match surface_status {
            wgpu::CurrentSurfaceTexture::Timeout => {
                warn!("Timed out while acquiring the next surface texture; skipping frame");
                Ok(())
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                warn!("Surface is occluded while acquiring the next texture; skipping frame");
                Ok(())
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                warn!("Recovering renderer surface after acquisition status: {surface_status:?}");
                self.ctx.resize(self.ctx.size)
            }
            wgpu::CurrentSurfaceTexture::Validation => Err(anyhow!(
                "Validation error while acquiring the next surface texture"
            )),
            wgpu::CurrentSurfaceTexture::Success(_)
            | wgpu::CurrentSurfaceTexture::Suboptimal(_) => Ok(()),
        }
    }

    fn size_from_dimensions(width: f64, height: f64) -> Size {
        Size {
            width: width.max(0.0).round() as u32,
            height: height.max(0.0).round() as u32,
        }
    }

    fn record_scene_pass_command_encoder(
        encoder: &mut CommandEncoder,
        render_mesh: &RenderMesh,
        render_pipeline: &RenderPipeline,
        queue: &Queue,
        model_binding_mode: ModelMatrixBindingMode,
        camera_bind_group: &BindGroup,
        light_bind_group: &BindGroup,
        view: &TextureView,
        depth_view: &TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Main Command Buffer"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
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
                view: depth_view,
                depth_ops: Some(Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(render_pipeline);
        Self::apply_model_matrix(&mut render_pass, render_mesh, queue, model_binding_mode);
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

    fn apply_model_matrix(
        render_pass: &mut wgpu::RenderPass<'_>,
        render_mesh: &RenderMesh,
        queue: &Queue,
        model_binding_mode: ModelMatrixBindingMode,
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
                render_pass.set_bind_group(2, model_bind_group, &[]);
            }
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
}

#[cfg(test)]
mod tests {
    use hyakou_core::types::Size;

    use crate::renderer::Renderer;

    #[test]
    fn test_size_from_dimensions_rounds_and_clamps_negative_values() {
        let size = Renderer::size_from_dimensions(640.6, -5.2);

        assert_eq!(
            size,
            Size {
                width: 641,
                height: 0,
            }
        );
    }
}
