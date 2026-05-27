use anyhow::{Result, anyhow};
use bytemuck::bytes_of;
use glam::{Vec3, Vec4};
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

pub(super) struct SceneGpuResources {
    camera_uniform: CameraUniform,
    camera_uniform_buffer: UniformBuffer,
    pub(super) camera_bind_group: BindGroup,
    light: LightSource,
    light_uniform_buffer: UniformBuffer,
    pub(super) light_bind_group: BindGroup,
    outline_uniform: OutlineUniform,
    outline_uniform_buffer: UniformBuffer,
    pub(super) outline_bind_group: BindGroup,
}

impl SceneGpuResources {
    // TODO: needs to be removed at some point // constants to build the buffers. they ain't doing shit
    const DEFAULT_LIGHT_POSITION: Vec3 = Vec3::new(0.0, 2.0, 1.0);
    const DEFAULT_LIGHT_COLOR: Vec3 = Vec3::new(1.0, 1.0, 1.0);
    const DEFAULT_OUTLINE_COLOR: Vec4 = Vec4::new(0.5, 0.1, 1.0, 1.0);
    const DEFAULT_OUTLINE_THICKNESS: f32 = 0.05;

    pub(super) fn new(ctx: &RenderContext, camera: &Camera) -> Result<Self> {
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update(camera);

        let camera_uniform_buffer = UniformBuffer::new(
            UniformBufferId::new("Camera".to_string()),
            &ctx.device,
            bytes_of(&camera_uniform),
            shared(Transform::default()),
        );
        let camera_bind_group = CameraUniform::bind_group(
            &ctx.device,
            &camera_uniform_buffer,
            &ctx.camera_bind_group_layout,
        );

        let mut light_transform = Transform::default();
        light_transform.translate(Self::DEFAULT_LIGHT_POSITION);
        let light_transform = shared(light_transform);
        let light = LightSource::new(light_transform.clone(), Self::DEFAULT_LIGHT_COLOR);
        let gpu_light_source = light
            .to_gpu()
            .ok_or_else(|| anyhow!("Failed to read default light transform"))?;
        let light_uniform_buffer = UniformBuffer::new(
            UniformBufferId::new("Light Uniform Buffer".to_string()),
            &ctx.device,
            bytes_of(&gpu_light_source),
            light_transform,
        );
        let light_bind_group = LightSource::bind_group(
            &ctx.device,
            &light_uniform_buffer,
            &ctx.light_bind_group_layout,
        );

        let outline_uniform =
            OutlineUniform::new(Self::DEFAULT_OUTLINE_COLOR, Self::DEFAULT_OUTLINE_THICKNESS);
        let outline_uniform_buffer = OutlineUniform::uniform_buffer(&ctx.device, &outline_uniform);
        let outline_bind_group = OutlineUniform::bind_group(
            &ctx.device,
            &outline_uniform_buffer,
            &ctx.outline_bind_group_layout,
        );

        Ok(Self {
            camera_uniform,
            camera_uniform_buffer,
            camera_bind_group,
            light,
            light_uniform_buffer,
            light_bind_group,
            outline_uniform,
            outline_uniform_buffer,
            outline_bind_group,
        })
    }

    pub(super) fn update(&mut self, queue: &Queue, camera: &Camera) {
        self.camera_uniform.update(camera);
        if let Some(gpu_light_source) = self.light.to_gpu() {
            self.light_uniform_buffer
                .update_buffer_transform(queue, bytes_of(&gpu_light_source))
                .unwrap()
        } else {
            warn!("Skipping light buffer - Transform in Light is still locked");
        }
        queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytes_of(&self.camera_uniform),
        );
    }

    pub(super) fn set_outline_color(&mut self, queue: &Queue, color: Vec4) {
        self.outline_uniform.color = color;
        queue.write_buffer(
            &self.outline_uniform_buffer,
            0,
            bytes_of(&self.outline_uniform),
        );
    }
}
