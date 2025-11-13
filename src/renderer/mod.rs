use std::sync::Arc;

use anyhow::Result;
use wgpu::{Color, CommandEncoder, CommandEncoderDescriptor, Operations, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, TextureView, TextureViewDescriptor};
use winit::{dpi::PhysicalPosition, window::Window};

use crate::renderer::{renderer_context::RendererContext, wrappers::WinitSurfaceProvider};

pub mod renderer_context;
pub mod geometry;
pub mod components;
pub mod util;
pub mod wrappers;
pub mod parameter;

pub struct Renderer {
    ctx: RendererContext,
    window: Arc<Window>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        Ok(Self {
            ctx: RendererContext::new(Some(WinitSurfaceProvider {
                window: window.clone(),
            }))
            .await
            .unwrap(),
            window,
        })
    }

    pub fn update(&mut self) {
        todo!()
    }

    pub fn render(&mut self, mouse_pos: PhysicalPosition<f64>) -> Result<()> {
        self.window.request_redraw();
        if self.ctx.surface_configuration.is_none() {
            return Ok(());
        }
        let output = self
            .ctx
            .surface
            .as_ref()
            .unwrap()
            .get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Rendering Encoder"),
            });
        self.record_command_encoder(&mut encoder, view, mouse_pos);
        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())  
    }

    fn record_command_encoder(&self,
         encoder: &mut CommandEncoder,
         view: TextureView,
         mouse_pos: PhysicalPosition<f64>) {
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
                        a: 0.2 
                    }),
                    store: wgpu::StoreOp::Store 
                }
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.ctx.depth_texture.view,
                depth_ops: Some(Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(&self.ctx.render_pipeline);
        render_pass.set_vertex_buffer(0, self.ctx.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &self.ctx.bind_group, &[]);
        render_pass.set_index_buffer(self.ctx.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.ctx.num_indices as u32, 0, 0..1);
    }
}

