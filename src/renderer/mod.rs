use std::{path::Path, sync::Arc, time::Instant};

use anyhow::Result;
use bytemuck::bytes_of;
use wgpu::{
    Color, CommandEncoder, CommandEncoderDescriptor, Operations, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, ShaderStages, TextureView,
    TextureViewDescriptor,
};
use winit::{
    dpi::PhysicalPosition,
    window::Window,
};

use crate::renderer::{
    components::{asset_manager::AssetManager, camera::CameraController, glTF::GLTFLoader},
    renderer_context::RenderContext,
    wrappers::WinitSurfaceProvider,
};

pub mod components;
pub mod geometry;
pub mod parameter;
pub mod renderer_context;
pub mod util;
pub mod wrappers;

const FRAME_TOTAL: u8 = 2;

pub struct Renderer {
    ctx: RenderContext,
    window: Arc<Window>,
    frame_idx: u8,
    pub camera_controller: CameraController,
    pub asset_manager: AssetManager,
    render_delta: Instant,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let ctx  =  RenderContext::new(Some(WinitSurfaceProvider {
                window: window.clone(),
            }))
            .await
            .unwrap();
        let mut asset_manager = AssetManager::new(ctx.device.clone());
        asset_manager.add_from_path("Monkey".to_string(), &Path::new("/Users/zapzap/Projects/hyako/assets/gltf/Suzanne.gltf" ));
        Ok(Self {
            ctx: ctx, 
            asset_manager,
            frame_idx: 0,
            camera_controller: CameraController::new(0.2),
            render_delta: Instant::now(),
            window,
        })
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.ctx.camera);
        self.ctx.camera_uniform.update(&self.ctx.camera);
        self.ctx.queue.write_buffer(&self.ctx.camera_uniform_buffer, 0, bytes_of(&self.ctx.camera_uniform));
    }

    pub fn render(
        &mut self,
        mouse_pos: PhysicalPosition<f64>,
    ) -> Result<()> {
        self.window.request_redraw();
        self.render_delta = Instant::now();
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
        let depth_texture = self.ctx.depth_texture.clone();
        self.record_scene_pass_command_encoder(&mut encoder, view, depth_texture.view, mouse_pos);
        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.frame_idx = (self.frame_idx + 1) % 1;
        Ok(())
    }

    fn record_scene_pass_command_encoder(
        &mut self,
        encoder: &mut CommandEncoder,
        view: TextureView,
        depth_view: TextureView,
        mouse_pos: PhysicalPosition<f64>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
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
                view: &depth_view,
                depth_ops: Some(Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
        });

        /*          depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
            view: &self.ctx.depth_texture.view,
            depth_ops: Some(Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }) , */
        render_pass.set_pipeline(&self.ctx.render_pipeline);
        render_pass.set_push_constants(
            ShaderStages::VERTEX,
            0,
            bytemuck::bytes_of(&self.ctx.model_matrix),
        );
        render_pass.set_vertex_buffer(0, self.ctx.vertex_buffer.slice(..));
        //  render_pass.set_bind_group(0, &self.ctx.mesh_bind_group, &[]);
        render_pass.set_bind_group(0, &self.ctx.camera_bind_group, &[]);
        render_pass.set_index_buffer(self.ctx.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.ctx.num_indices as u32, 0, 0..1);
    }
}
