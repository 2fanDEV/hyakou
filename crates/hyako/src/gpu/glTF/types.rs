use glam::Vec4;
use hyakou_core::geometry::node::NodeGraph;

pub struct ImportedScene {
    pub node_graph: NodeGraph,
    pub materials: Vec<ImportedMaterial>,
    pub images: Vec<ImportedImage>,
    pub textures: Vec<ImportedTexture>,
    pub samplers: Vec<ImportedSampler>,
}

impl ImportedScene {
    pub fn new(
        node_graph: NodeGraph,
        materials: Vec<ImportedMaterial>,
        images: Vec<ImportedImage>,
        textures: Vec<ImportedTexture>,
        samplers: Vec<ImportedSampler>,
    ) -> Self {
        Self {
            node_graph,
            materials,
            images,
            textures,
            samplers,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportedMaterial {
    pub index: usize,
    pub name: Option<String>,
    pub base_color_factor: Vec4,
    pub base_color_texture: Option<ImportedTextureRef>,
    pub alpha_mode: ImportedAlphaMode,
    pub alpha_cutoff: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImportedAlphaMode {
    Opaque,
    Mask,
    Blend,
}

#[derive(Debug, Clone, Copy)]
pub struct ImportedTextureRef {
    pub texture_index: usize,
    pub tex_coord: u32,
}

#[derive(Debug, Clone)]
pub struct ImportedImage {
    pub index: usize,
    pub name: Option<String>,
    pub width: u32,
    pub height: u32,
    pub pixels_rgba8: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ImportedTexture {
    pub index: usize,
    pub name: Option<String>,
    pub image_index: usize,
    pub sampler_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ImportedSampler {
    pub index: usize,
    pub name: Option<String>,
    pub mag_filter: ImportedMagFilter,
    pub min_filter: ImportedMinFilter,
    pub wrap_s: ImportedWrapMode,
    pub wrap_t: ImportedWrapMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportedMagFilter {
    Nearest,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportedMinFilter {
    Nearest,
    Linear,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportedWrapMode {
    ClampToEdge,
    MirroredRepeat,
    Repeat,
}
