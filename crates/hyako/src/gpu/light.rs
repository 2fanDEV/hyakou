use wgpu::ShaderStages;

use hyakou_core::components::light::GpuLightSource;

use crate::gpu::uniform::GpuUniformMetadata;

impl GpuUniformMetadata for GpuLightSource {
    const LAYOUT_LABEL: &'static str = "Light Source";
    const BIND_GROUP_LABEL: &'static str = "Light Bind Group";
    const VISIBILITY: ShaderStages = ShaderStages::VERTEX_FRAGMENT;
}
