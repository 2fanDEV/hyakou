use crate::gpu::uniform::GpuUniformMetadata;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use wgpu::ShaderStages;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct ModelMatrixUniform {
    pub model_matrix: Mat4,
}

impl ModelMatrixUniform {
    pub fn new(model_matrix: Mat4) -> Self {
        Self { model_matrix }
    }
}

impl GpuUniformMetadata for ModelMatrixUniform {
    const LAYOUT_LABEL: &'static str = "Model Matrix Buffer";
    const BIND_GROUP_LABEL: &'static str = "Model Matrix Bind Group";
    const VISIBILITY: ShaderStages = ShaderStages::VERTEX;
}
