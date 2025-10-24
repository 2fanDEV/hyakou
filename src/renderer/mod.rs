use std::sync::Arc;

use anyhow::Result;
use wgpu::{CommandEncoderDescriptor, TextureViewDescriptor};
use winit::window::Window;

use crate::renderer::{renderer_context::RendererContext, wrappers::WinitSurfaceProvider};

pub mod renderer_context;
pub mod util;
pub mod wrappers;
pub struct Renderer {
    context: RendererContext,
    window: Arc<Window>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        Ok(Self {
            context: RendererContext::new(Some(WinitSurfaceProvider {
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

    pub fn render(&mut self) -> Result<()> {
        self.window.request_redraw();
        if self.context.surface_configuration.is_none() {
            return Ok(());
        }
        let output = self
            .context
            .surface
            .as_ref()
            .unwrap()
            .get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let encoder = self
            .context
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Rendering Encoder"),
            });

        /* encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: (),
            color_attachments: (),
            depth_stencil_attachment: (),
            timestamp_writes: (),
            occlusion_query_set: (),
        }); */

        Ok(())
    }
}
