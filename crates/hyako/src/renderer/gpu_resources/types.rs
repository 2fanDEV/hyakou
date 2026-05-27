use anyhow::{Result, anyhow};
use bytemuck::bytes_of;
use glam::Vec4;
use hyakou_core::{
    components::{camera::camera::Camera, light::LightSource},
    traits::BindGroupProvider,
    types::{TransformBuffer, ids::UniformBufferId, transform::Transform},
};
use log::warn;
use shared::shared;
use wgpu::{BindGroup, Queue};

use crate::{
    gpu::{
        buffers::{camera_buffer::CameraUniform, uniform::UniformBuffer},
        outline::OutlineUniform,
    },
    renderer::renderer_context::RenderContext,
};

pub(super) struct CameraGpuResources {
    uniform: CameraUniform,
    uniform_buffer: UniformBuffer,
    bind_group: BindGroup,
}

impl CameraGpuResources {
    pub(super) fn new(ctx: &RenderContext, camera: &Camera) -> Self {
        let mut uniform = CameraUniform::new();
        uniform.update(camera);

        let uniform_buffer = UniformBuffer::new(
            UniformBufferId::new("Camera".to_string()),
            &ctx.device,
            bytes_of(&uniform),
            shared(Transform::default()),
        );
        let bind_group = CameraUniform::bind_group(
            &ctx.device,
            &uniform_buffer,
            &ctx.camera_bind_group_layout,
        );

        Self {
            uniform,
            uniform_buffer,
            bind_group,
        }
    }

    pub(super) fn update(&mut self, queue: &Queue, camera: &Camera) {
        self.uniform.update(camera);
        queue.write_buffer(&self.uniform_buffer, 0, bytes_of(&self.uniform));
    }

    pub(super) fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

pub(super) struct LightGpuResources {
    source: LightSource,
    uniform_buffer: UniformBuffer,
    bind_group: BindGroup,
}

impl LightGpuResources {
    pub(super) fn new(ctx: &RenderContext, source: LightSource) -> Result<Self> {
        let gpu_light_source = source
            .to_gpu()
            .ok_or_else(|| anyhow!("Failed to read light transform"))?;
        let uniform_buffer = UniformBuffer::new(
            UniformBufferId::new("Light Uniform Buffer".to_string()),
            &ctx.device,
            bytes_of(&gpu_light_source),
            source.transform.clone(),
        );
        let bind_group = LightSource::bind_group(
            &ctx.device,
            &uniform_buffer,
            &ctx.light_bind_group_layout,
        );

        Ok(Self {
            source,
            uniform_buffer,
            bind_group,
        })
    }

    pub(super) fn update(&mut self, queue: &Queue) {
        if let Some(gpu_light_source) = self.source.to_gpu() {
            self.uniform_buffer
                .update_buffer_transform(queue, bytes_of(&gpu_light_source))
                .unwrap()
        } else {
            warn!("Skipping light buffer - Transform in Light is still locked");
        }
    }

    pub(super) fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

pub(super) struct OutlineGpuResources {
    uniform: OutlineUniform,
    uniform_buffer: UniformBuffer,
    bind_group: BindGroup,
}

impl OutlineGpuResources {
    pub(super) fn new(ctx: &RenderContext, color: Vec4, thickness: f32) -> Self {
        let uniform = OutlineUniform::new(color, thickness);
        let uniform_buffer = OutlineUniform::uniform_buffer(&ctx.device, &uniform);
        let bind_group = OutlineUniform::bind_group(
            &ctx.device,
            &uniform_buffer,
            &ctx.outline_bind_group_layout,
        );

        Self {
            uniform,
            uniform_buffer,
            bind_group,
        }
    }

    pub(super) fn set_color(&mut self, queue: &Queue, color: Vec4) {
        self.uniform.color = color;
        queue.write_buffer(&self.uniform_buffer, 0, bytes_of(&self.uniform));
    }

    pub(super) fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
