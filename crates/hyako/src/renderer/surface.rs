use anyhow::{Result, anyhow};
use hyakou_core::types::Size;
use log::warn;
use wgpu::{Device, Surface, SurfaceConfiguration, TextureFormat, TextureUsages};

use crate::{gpu::texture::Texture, renderer::renderer_context::RenderContext};

impl RenderContext {
    pub fn resize(&mut self, size: Size) -> Result<()> {
        self.size = size;

        if size.is_zero() {
            warn!(
                "Ignoring resize because wgpu surfaces cannot be configured with zero width or height: {}x{}",
                size.width, size.height
            );
            return Ok(());
        }

        let Some(surface) = self.surface.as_ref() else {
            self.depth_texture =
                Texture::create_depth_texture(Self::DEPTH_TEXTURE_LABEL, &self.device, &self.size);
            return Ok(());
        };

        let Some(surface_configuration) = self.surface_configuration.as_mut() else {
            return Err(anyhow!(
                "Cannot resize render surface because the surface configuration is missing"
            ));
        };

        surface_configuration.width = size.width;
        surface_configuration.height = size.height;
        surface.configure(&self.device, surface_configuration);
        self.depth_texture =
            Texture::create_depth_texture(Self::DEPTH_TEXTURE_LABEL, &self.device, &self.size);

        Ok(())
    }
}

pub(super) fn surface_format(
    surface_configuration: Option<&SurfaceConfiguration>,
) -> TextureFormat {
    surface_configuration
        .map(|configuration| configuration.format)
        .unwrap_or(TextureFormat::Bgra8UnormSrgb)
}

pub(super) fn init_surface_configuration(
    surface: Option<&Surface<'static>>,
    adapter: &wgpu::Adapter,
    size: Size,
    device: &Device,
) -> Option<SurfaceConfiguration> {
    let Some(surface) = surface else {
        return None;
    };

    let capabilities = surface.get_capabilities(adapter);
    let format = capabilities
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(capabilities.formats[0]);

    let configured_size = size.clamp_size_for_gpu();

    let surface_configuration = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format,
        width: configured_size.width,
        height: configured_size.height,
        present_mode: capabilities.present_modes[0],
        desired_maximum_frame_latency: 2,
        alpha_mode: capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    if !size.is_zero() {
        surface.configure(device, &surface_configuration);
    }

    Some(surface_configuration)
}
