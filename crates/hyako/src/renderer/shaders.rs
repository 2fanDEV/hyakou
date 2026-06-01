use hyakou_core::types::ModelMatrixBindingMode;
use wgpu::{Device, include_wgsl};

pub(super) fn create_light_shader_module(
    device: &Device,
    model_binding_mode: ModelMatrixBindingMode,
) -> wgpu::ShaderModule {
    match model_binding_mode {
        ModelMatrixBindingMode::Immediate => {
            device.create_shader_module(include_wgsl!("../../assets/vertex.wgsl"))
        }
        ModelMatrixBindingMode::Uniform => {
            device.create_shader_module(include_wgsl!("../../assets/vertex_uniform.wgsl"))
        }
    }
}

pub(super) fn create_no_light_shader_module(
    device: &Device,
    model_binding_mode: ModelMatrixBindingMode,
) -> wgpu::ShaderModule {
    match model_binding_mode {
        ModelMatrixBindingMode::Immediate => {
            device.create_shader_module(include_wgsl!("../../assets/no_light_vertex.wgsl"))
        }
        ModelMatrixBindingMode::Uniform => {
            device.create_shader_module(include_wgsl!("../../assets/no_light_vertex_uniform.wgsl"))
        }
    }
}

pub(super) fn create_outline_shader_module(
    device: &Device,
    model_binding_mode: ModelMatrixBindingMode,
) -> wgpu::ShaderModule {
    match model_binding_mode {
        ModelMatrixBindingMode::Immediate => {
            device.create_shader_module(include_wgsl!("../../assets/outline.wgsl"))
        }
        ModelMatrixBindingMode::Uniform => {
            device.create_shader_module(include_wgsl!("../../assets/outline_uniform.wgsl"))
        }
    }
}
