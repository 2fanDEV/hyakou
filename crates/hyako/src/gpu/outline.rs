use bytemuck::{Pod, Zeroable};
use glam::Vec4;
use wgpu::ShaderStages;

use crate::gpu::uniform::GpuUniformMetadata;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct OutlineUniform {
    pub color: Vec4,
    pub thickness: f32,
    _padding: [f32; 3],
}

impl OutlineUniform {
    pub fn new(color: Vec4, thickness: f32) -> Self {
        Self {
            color,
            thickness,
            _padding: [0.0; 3],
        }
    }
}

impl GpuUniformMetadata for OutlineUniform {
    const LAYOUT_LABEL: &'static str = "Outline Bind Group Layout";
    const BIND_GROUP_LABEL: &'static str = "Outline Bind Group";
    const VISIBILITY: ShaderStages = ShaderStages::VERTEX_FRAGMENT;
}
