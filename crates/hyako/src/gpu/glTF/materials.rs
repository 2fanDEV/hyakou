use anyhow::{Result, anyhow};
use glam::Vec4;

use super::types::{
    ImportedAlphaMode, ImportedMagFilter, ImportedMaterial, ImportedMinFilter, ImportedSampler,
    ImportedTexture, ImportedTextureRef, ImportedWrapMode,
};

pub(super) fn load_materials(gltf: &gltf::Gltf) -> Result<Vec<ImportedMaterial>> {
    gltf.materials().map(import_material).collect()
}

pub(super) fn load_textures(gltf: &gltf::Gltf) -> Vec<ImportedTexture> {
    gltf.textures().map(import_texture).collect()
}

pub(super) fn load_samplers(gltf: &gltf::Gltf) -> Vec<ImportedSampler> {
    gltf.samplers().map(import_sampler).collect()
}

fn import_material(material: gltf::Material<'_>) -> Result<ImportedMaterial> {
    let material_index = material.index().ok_or_else(|| {
        anyhow!("glTF default material should not appear in the explicit material iterator")
    })?;
    let pbr = material.pbr_metallic_roughness();
    let base_color_factor = pbr.base_color_factor();

    Ok(ImportedMaterial {
        index: material_index,
        name: material.name().map(str::to_owned),
        base_color_factor: Vec4::new(
            base_color_factor[0],
            base_color_factor[1],
            base_color_factor[2],
            base_color_factor[3],
        ),
        base_color_texture: pbr
            .base_color_texture()
            .map(import_texture_ref)
            .transpose()?,
        alpha_mode: import_alpha_mode(material.alpha_mode()),
        alpha_cutoff: material.alpha_cutoff(),
    })
}

fn import_texture(texture: gltf::Texture<'_>) -> ImportedTexture {
    ImportedTexture {
        index: texture.index(),
        name: texture.name().map(str::to_owned),
        image_index: texture.source().index(),
        sampler_index: texture.sampler().index(),
    }
}

fn import_sampler(sampler: gltf::texture::Sampler<'_>) -> ImportedSampler {
    let sampler_index = sampler
        .index()
        .expect("glTF default sampler should not appear in the explicit sampler iterator");

    ImportedSampler {
        index: sampler_index,
        name: sampler.name().map(str::to_owned),
        mag_filter: sampler
            .mag_filter()
            .map(import_mag_filter)
            .unwrap_or(ImportedMagFilter::Linear),
        min_filter: sampler
            .min_filter()
            .map(import_min_filter)
            .unwrap_or(ImportedMinFilter::LinearMipmapLinear),
        wrap_s: import_wrap_mode(sampler.wrap_s()),
        wrap_t: import_wrap_mode(sampler.wrap_t()),
    }
}

fn import_texture_ref(info: gltf::texture::Info<'_>) -> Result<ImportedTextureRef> {
    if info.tex_coord() != 0 {
        return Err(anyhow!(
            "Unsupported base color texture coordinate set `TEXCOORD_{}`; only `TEXCOORD_0` is supported",
            info.tex_coord()
        ));
    }

    Ok(ImportedTextureRef {
        texture_index: info.texture().index(),
        tex_coord: info.tex_coord(),
    })
}

fn import_alpha_mode(alpha_mode: gltf::material::AlphaMode) -> ImportedAlphaMode {
    match alpha_mode {
        gltf::material::AlphaMode::Opaque => ImportedAlphaMode::Opaque,
        gltf::material::AlphaMode::Mask => ImportedAlphaMode::Mask,
        gltf::material::AlphaMode::Blend => ImportedAlphaMode::Blend,
    }
}

fn import_mag_filter(filter: gltf::texture::MagFilter) -> ImportedMagFilter {
    match filter {
        gltf::texture::MagFilter::Nearest => ImportedMagFilter::Nearest,
        gltf::texture::MagFilter::Linear => ImportedMagFilter::Linear,
    }
}

fn import_min_filter(filter: gltf::texture::MinFilter) -> ImportedMinFilter {
    match filter {
        gltf::texture::MinFilter::Nearest => ImportedMinFilter::Nearest,
        gltf::texture::MinFilter::Linear => ImportedMinFilter::Linear,
        gltf::texture::MinFilter::NearestMipmapNearest => ImportedMinFilter::NearestMipmapNearest,
        gltf::texture::MinFilter::LinearMipmapNearest => ImportedMinFilter::LinearMipmapNearest,
        gltf::texture::MinFilter::NearestMipmapLinear => ImportedMinFilter::NearestMipmapLinear,
        gltf::texture::MinFilter::LinearMipmapLinear => ImportedMinFilter::LinearMipmapLinear,
    }
}

fn import_wrap_mode(mode: gltf::texture::WrappingMode) -> ImportedWrapMode {
    match mode {
        gltf::texture::WrappingMode::ClampToEdge => ImportedWrapMode::ClampToEdge,
        gltf::texture::WrappingMode::MirroredRepeat => ImportedWrapMode::MirroredRepeat,
        gltf::texture::WrappingMode::Repeat => ImportedWrapMode::Repeat,
    }
}
