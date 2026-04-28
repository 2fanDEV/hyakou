use anyhow::{Result, anyhow};
use hyakou_core::types::Size;
use log::warn;
use wgpu::{CommandEncoderDescriptor, TextureViewDescriptor};
use winit::window::Window;

use crate::renderer::{frame::SurfaceFrame, renderer_context::RenderContext};

pub struct SurfaceFrameController;

impl SurfaceFrameController {
    pub fn new() -> Self {
        Self
    }

    pub fn begin_frame(
        &mut self,
        window: &Window,
        ctx: &mut RenderContext,
    ) -> Result<Option<SurfaceFrame>> {
        window.request_redraw();
        if ctx.surface_configuration.is_none() || ctx.size.is_zero() {
            return Ok(None);
        }

        let (output, should_reconfigure_surface) =
            match ctx.surface.as_ref().unwrap().get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(output) => (output, false),
                wgpu::CurrentSurfaceTexture::Suboptimal(output) => (output, true),
                surface_status => {
                    self.handle_surface_acquisition_status(ctx, surface_status)?;
                    return Ok(None);
                }
            };

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let encoder = ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Rendering Encoder"),
            });
        let depth_view = ctx.depth_texture.view.clone();
        let size_in_pixels = [ctx.size.width, ctx.size.height];

        Ok(Some(SurfaceFrame::new(
            output,
            encoder,
            ctx.queue.clone(),
            view,
            depth_view,
            size_in_pixels,
            should_reconfigure_surface,
        )))
    }

    pub fn finish_frame(&mut self, ctx: &mut RenderContext, frame: SurfaceFrame) -> Result<()> {
        if frame.finish() {
            ctx.resize(ctx.size)?;
        }

        Ok(())
    }

    pub fn resize(&mut self, ctx: &mut RenderContext, size: Size) -> Result<()> {
        ctx.resize(size)
    }

    pub fn size_from_dimensions(width: f64, height: f64) -> Size {
        Size {
            width: width.max(0.0).round() as u32,
            height: height.max(0.0).round() as u32,
        }
    }

    fn handle_surface_acquisition_status(
        &mut self,
        ctx: &mut RenderContext,
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
                ctx.resize(ctx.size)
            }
            wgpu::CurrentSurfaceTexture::Validation => Err(anyhow!(
                "Validation error while acquiring the next surface texture"
            )),
            wgpu::CurrentSurfaceTexture::Success(_)
            | wgpu::CurrentSurfaceTexture::Suboptimal(_) => Ok(()),
        }
    }
}

impl Default for SurfaceFrameController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use hyakou_core::types::Size;

    use crate::renderer::surface_frame_controller::SurfaceFrameController;

    #[test]
    fn test_size_from_dimensions_rounds_and_clamps_negative_values() {
        let size = SurfaceFrameController::size_from_dimensions(640.6, -5.2);

        assert_eq!(
            size,
            Size {
                width: 641,
                height: 0,
            }
        );
    }
}
