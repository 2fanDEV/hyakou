mod types;

use anyhow::Result;
use glam::Vec4;
use hyakou_core::components::{camera::camera::Camera, light::LightSource};
use wgpu::{BindGroup, Queue};

use crate::renderer::renderer_context::RenderContext;

use types::{CameraGpuResources, LightGpuResources, OutlineGpuResources};

const DEFAULT_OUTLINE_COLOR: Vec4 = Vec4::new(0.5, 0.1, 1.0, 1.0);
const DEFAULT_OUTLINE_THICKNESS: f32 = 0.05;

pub(super) struct SceneGpuResources {
    camera: CameraGpuResources,
    light: Option<LightGpuResources>,
    outline: Option<OutlineGpuResources>,
}

impl SceneGpuResources {
    pub(super) fn new(ctx: &RenderContext, camera: &Camera) -> Result<Self> {
        Ok(Self {
            camera: CameraGpuResources::new(ctx, camera),
            light: None,
            outline: Some(OutlineGpuResources::new(
                ctx,
                DEFAULT_OUTLINE_COLOR,
                DEFAULT_OUTLINE_THICKNESS,
            )),
        })
    }

    pub(super) fn update(&mut self, queue: &Queue, camera: &Camera) {
        self.camera.update(queue, camera);
        if let Some(light) = self.light.as_mut() {
            light.update(queue);
        }
    }

    pub(super) fn set_light(&mut self, ctx: &RenderContext, light: LightSource) -> Result<()> {
        self.light = Some(LightGpuResources::new(ctx, light)?);
        Ok(())
    }

    pub(super) fn set_outline(&mut self, ctx: &RenderContext, color: Vec4, thickness: f32) {
        self.outline = Some(OutlineGpuResources::new(ctx, color, thickness));
    }

    pub(super) fn set_outline_color(&mut self, queue: &Queue, color: Vec4) {
        if let Some(outline) = self.outline.as_mut() {
            outline.set_color(queue, color);
        }
    }

    pub(super) fn camera_bind_group(&self) -> &BindGroup {
        self.camera.bind_group()
    }

    pub(super) fn light_bind_group(&self) -> Option<&BindGroup> {
        self.light.as_ref().map(|light| light.bind_group())
    }

    pub(super) fn outline_bind_group(&self) -> Option<&BindGroup> {
        self.outline.as_ref().map(|outline| outline.bind_group())
    }
}
