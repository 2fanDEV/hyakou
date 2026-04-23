use hyakou_core::types::Size;
use wgpu::{
    CompareFunction, Device, Extent3d, FilterMode, MipmapFilterMode, Sampler, SamplerDescriptor,
    TextureDescriptor, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
    util::{DeviceExt, TextureDataOrder},
};

#[derive(Debug, Clone)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;
    pub const COLOR_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

    pub fn create_depth_texture(label: &str, device: &Device, size: &Size) -> Texture {
        let size = size.clamp_size_for_gpu();
        let extent_size = Extent3d {
            width: size.width,
            height: size.height,
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
            mipmap_filter: MipmapFilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: Some(CompareFunction::Less),

            ..Default::default()
        });

        Texture {
            texture,
            view,
            sampler,
        }
    }

    pub fn create_color_texture(
        label: &str,
        device: &Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        rgba8_pixels: &[u8],
        sampler_descriptor: SamplerDescriptor<'_>,
    ) -> Texture {
        let texture = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some(label),
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: Self::COLOR_FORMAT,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            TextureDataOrder::LayerMajor,
            rgba8_pixels,
        );
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&sampler_descriptor);

        Texture {
            texture,
            view,
            sampler,
        }
    }
}
