use hyakou_core::{components::light::GpuLightSource, types::ModelMatrixBindingMode};
use wgpu::{BindGroupLayout, Device, RenderPipeline, TextureFormat};

use crate::{
    gpu::{
        buffers::{camera_buffer::CameraUniform, model_matrix::ModelMatrixUniform},
        material::GpuMaterial,
        outline::OutlineUniform,
        render_pipeline::{create_outline_render_pipeline, create_render_pipeline},
        uniform::BindGroupProvider,
    },
    renderer::{renderer_context::RenderContext, shaders},
};

pub(super) struct PipelineResources {
    pub(super) camera_bind_group_layout: BindGroupLayout,
    pub(super) light_bind_group_layout: BindGroupLayout,
    pub(super) model_bind_group_layout: Option<BindGroupLayout>,
    pub(super) material_bind_group_layout: BindGroupLayout,
    pub(super) outline_bind_group_layout: BindGroupLayout,
    pub(super) light_render_pipeline: RenderPipeline,
    pub(super) no_light_render_pipeline: RenderPipeline,
    pub(super) outline_render_pipeline: RenderPipeline,
}

pub(super) fn create_pipeline_resources(
    device: &Device,
    model_binding_mode: ModelMatrixBindingMode,
    format: TextureFormat,
) -> PipelineResources {
    let camera_bind_group_layout = CameraUniform::bind_group_layout(device);
    let light_bind_group_layout = GpuLightSource::bind_group_layout(device);
    let model_bind_group_layout = (model_binding_mode == ModelMatrixBindingMode::Uniform)
        .then(|| ModelMatrixUniform::bind_group_layout(device));
    let material_bind_group_layout = GpuMaterial::bind_group_layout(device);
    let outline_bind_group_layout = OutlineUniform::bind_group_layout(device);

    let render_pipeline_layout = create_render_pipeline_layout(
        device,
        model_binding_mode,
        &camera_bind_group_layout,
        &light_bind_group_layout,
        model_bind_group_layout.as_ref(),
        &material_bind_group_layout,
    );
    let outline_pipeline_layout = create_outline_pipeline_layout(
        device,
        model_binding_mode,
        &camera_bind_group_layout,
        model_bind_group_layout.as_ref(),
        &outline_bind_group_layout,
    );

    let no_light_render_pipeline = create_render_pipeline(
        device,
        "no light render pass",
        &render_pipeline_layout,
        format,
        shaders::create_no_light_shader_module(device, model_binding_mode),
        Some(TextureFormat::Depth32Float),
    );
    let light_render_pipeline = create_render_pipeline(
        device,
        "light render pass",
        &render_pipeline_layout,
        format,
        shaders::create_light_shader_module(device, model_binding_mode),
        Some(TextureFormat::Depth32Float),
    );
    let outline_render_pipeline = create_outline_render_pipeline(
        device,
        &outline_pipeline_layout,
        format,
        shaders::create_outline_shader_module(device, model_binding_mode),
        TextureFormat::Depth32Float,
    );

    PipelineResources {
        camera_bind_group_layout,
        light_bind_group_layout,
        model_bind_group_layout,
        material_bind_group_layout,
        outline_bind_group_layout,
        light_render_pipeline,
        no_light_render_pipeline,
        outline_render_pipeline,
    }
}

fn create_render_pipeline_layout(
    device: &Device,
    model_binding_mode: ModelMatrixBindingMode,
    camera_bind_group_layout: &BindGroupLayout,
    light_bind_group_layout: &BindGroupLayout,
    model_bind_group_layout: Option<&BindGroupLayout>,
    material_bind_group_layout: &BindGroupLayout,
) -> wgpu::PipelineLayout {
    let bind_group_layouts = if let Some(model_bind_group_layout) = model_bind_group_layout {
        vec![
            Some(camera_bind_group_layout),
            Some(light_bind_group_layout),
            Some(model_bind_group_layout),
            Some(material_bind_group_layout),
        ]
    } else {
        vec![
            Some(camera_bind_group_layout),
            Some(light_bind_group_layout),
            Some(material_bind_group_layout),
        ]
    };

    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &bind_group_layouts,
        immediate_size: immediate_size(model_binding_mode),
    })
}

fn create_outline_pipeline_layout(
    device: &Device,
    model_binding_mode: ModelMatrixBindingMode,
    camera_bind_group_layout: &BindGroupLayout,
    model_bind_group_layout: Option<&BindGroupLayout>,
    outline_bind_group_layout: &BindGroupLayout,
) -> wgpu::PipelineLayout {
    let bind_group_layouts = if let Some(model_bind_group_layout) = model_bind_group_layout {
        vec![
            Some(camera_bind_group_layout),
            Some(model_bind_group_layout),
            Some(outline_bind_group_layout),
        ]
    } else {
        vec![
            Some(camera_bind_group_layout),
            Some(outline_bind_group_layout),
        ]
    };

    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Outline Pipeline Layout"),
        bind_group_layouts: &bind_group_layouts,
        immediate_size: immediate_size(model_binding_mode),
    })
}

fn immediate_size(model_binding_mode: ModelMatrixBindingMode) -> u32 {
    if model_binding_mode == ModelMatrixBindingMode::Immediate {
        RenderContext::IMMEDIATE_MODEL_MATRIX_SIZE
    } else {
        0
    }
}
