use std::iter::Filter;

use gltf::json::texture::CLAMP_TO_EDGE;
use wgpu::{CompareFunction, Device, Extent3d, FilterMode, Sampler, SamplerDescriptor, SurfaceConfiguration, TextureDescriptor, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, naga::back::msl::sampler::CompareFunc};

#[derive(Debug, Clone)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32FloatStencil8;

    pub fn create_depth_texture(
        label: &str,
        device: &Device,
        config: &SurfaceConfiguration,
    ) -> Texture {
        let width = config.width;
        let height = config.height;
        let extent_size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let desc = TextureDescriptor {
            label: Some(label),
            size: extent_size,
            mip_level_count: 1, 
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some(label),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0, 
            compare: Some(CompareFunction::Less),
            ..Default::default()
        });

        Texture {
            texture,
            view,
            sampler
        }
    }
}
