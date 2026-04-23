use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use hyakou_core::{
    shared,
    types::{ids::UniformBufferId, transform::Transform},
};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BufferBinding,
    Device, FilterMode, MipmapFilterMode, SamplerBindingType, ShaderStages, TextureSampleType,
    TextureViewDimension,
};

use crate::gpu::{
    buffers::uniform::UniformBuffer,
    glTF::{
        ImportedMagFilter, ImportedMaterial, ImportedMinFilter, ImportedSampler, ImportedWrapMode,
    },
    texture::Texture,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MaterialUniform {
    pub base_color_factor: [f32; 4],
}

#[derive(Debug, Clone)]
pub struct GpuMaterial {
    pub uniform_buffer: UniformBuffer,
    pub bind_group: BindGroup,
    pub texture: Rc<Texture>,
}

impl MaterialUniform {
    pub fn new(base_color_factor: [f32; 4]) -> Self {
        Self { base_color_factor }
    }
}

impl GpuMaterial {
    pub fn bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Material Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    pub fn new(
        device: &Device,
        bind_group_layout: &BindGroupLayout,
        label: &str,
        material: &ImportedMaterial,
        texture: Rc<Texture>,
    ) -> Self {
        let uniform = MaterialUniform::new(material.base_color_factor.to_array());
        let uniform_buffer = UniformBuffer::new(
            UniformBufferId::new(format!("Material Uniform Buffer: {label}")),
            device,
            bytemuck::bytes_of(&uniform),
            shared(Transform::default()),
        );
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some(label),
            layout: bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&texture.view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&texture.sampler),
                },
            ],
        });

        Self {
            uniform_buffer,
            bind_group,
            texture,
        }
    }
}

pub fn default_sampler_descriptor(label: &str) -> wgpu::SamplerDescriptor<'_> {
    wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        address_mode_w: AddressMode::Repeat,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: MipmapFilterMode::Linear,
        ..Default::default()
    }
}

pub fn sampler_descriptor_from_imported_sampler<'a>(
    sampler: &'a ImportedSampler,
    label: &'a str,
) -> wgpu::SamplerDescriptor<'a> {
    let (min_filter, mipmap_filter) = imported_min_filter_to_wgpu(sampler.min_filter);

    wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: imported_wrap_mode_to_wgpu(sampler.wrap_s),
        address_mode_v: imported_wrap_mode_to_wgpu(sampler.wrap_t),
        address_mode_w: AddressMode::Repeat,
        mag_filter: imported_mag_filter_to_wgpu(sampler.mag_filter),
        min_filter,
        mipmap_filter,
        ..Default::default()
    }
}

fn imported_wrap_mode_to_wgpu(mode: ImportedWrapMode) -> AddressMode {
    match mode {
        ImportedWrapMode::ClampToEdge => AddressMode::ClampToEdge,
        ImportedWrapMode::MirroredRepeat => AddressMode::MirrorRepeat,
        ImportedWrapMode::Repeat => AddressMode::Repeat,
    }
}

fn imported_mag_filter_to_wgpu(filter: ImportedMagFilter) -> FilterMode {
    match filter {
        ImportedMagFilter::Nearest => FilterMode::Nearest,
        ImportedMagFilter::Linear => FilterMode::Linear,
    }
}

fn imported_min_filter_to_wgpu(filter: ImportedMinFilter) -> (FilterMode, MipmapFilterMode) {
    match filter {
        ImportedMinFilter::Nearest => (FilterMode::Nearest, MipmapFilterMode::Nearest),
        ImportedMinFilter::Linear => (FilterMode::Linear, MipmapFilterMode::Nearest),
        ImportedMinFilter::NearestMipmapNearest => (FilterMode::Nearest, MipmapFilterMode::Nearest),
        ImportedMinFilter::LinearMipmapNearest => (FilterMode::Linear, MipmapFilterMode::Nearest),
        ImportedMinFilter::NearestMipmapLinear => (FilterMode::Nearest, MipmapFilterMode::Linear),
        ImportedMinFilter::LinearMipmapLinear => (FilterMode::Linear, MipmapFilterMode::Linear),
    }
}
