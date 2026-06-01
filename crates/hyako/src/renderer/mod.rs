use std::sync::RwLock;
use std::{collections::HashMap, rc::Rc, sync::Arc};

use crate::{
    flow::FrameComposer,
    gpu::render_mesh::RenderMesh,
    gui::EguiRenderer,
    renderer::{renderer_context::RenderContext, surface_frame_controller::SurfaceFrameController},
};
use anyhow::Result;
use glam::Vec4;
use hyakou_core::{
    animations::Animator,
    components::{camera::camera::Camera, light::LightSource},
    types::{DeltaTime64, Size, ids::MeshId},
};
use log::error;
use wgpu::{Device, TextureFormat};
use winit::window::Window;

pub mod actions;
pub mod frame;
mod gpu_resources;
pub mod handlers;
mod pipelines;
mod render_pass;
pub mod renderer_context;
mod shaders;
mod surface;
pub mod surface_frame_controller;
pub mod wrappers;

use gpu_resources::SceneGpuResources;

pub struct SceneRenderInput<'a> {
    pub normal_meshes: &'a [Rc<RenderMesh>],
    pub light_meshes: &'a [Rc<RenderMesh>],
    pub outlined_meshes: &'a [Rc<RenderMesh>],
}

pub struct SceneRenderer {
    inner: RwLock<SceneRendererInner>,
}

struct SceneRendererInner {
    ctx: RenderContext,
    gpu_resources: SceneGpuResources,
    animators: HashMap<MeshId, Animator>,
}

impl SceneRenderer {
    pub fn from_context(ctx: RenderContext, camera: &Camera) -> Result<Self> {
        let gpu_resources = SceneGpuResources::new(&ctx, camera)?;

        Ok(Self {
            inner: RwLock::new(SceneRendererInner {
                ctx,
                gpu_resources,
                animators: HashMap::new(),
            }),
        })
    }

    pub fn render_surface_frame(
        &self,
        surface_frame_controller: &mut SurfaceFrameController,
        window: &Window,
        frame_composer: &mut FrameComposer,
        egui_renderer: Option<&mut EguiRenderer>,
        dt: f64,
        camera: &Camera,
        render_input: SceneRenderInput<'_>,
    ) -> Result<()> {
        let mut inner = self.inner.write().expect("SceneRenderer lock poisoned");
        inner.update_gpu(dt, camera);

        let Some(mut frame) = surface_frame_controller.begin_frame(window, &mut inner.ctx)? else {
            return Ok(());
        };

        let mut egui_renderer = egui_renderer;
        {
            let mut target = frame.target();
            inner.render_scene(&mut target, render_input);
            frame_composer.compose_frame(
                &mut target,
                egui_renderer.as_mut().map(|renderer| &mut **renderer),
            );
        }

        let finish_result = surface_frame_controller.finish_frame(&mut inner.ctx, frame);

        if let Some(egui_renderer) = egui_renderer.as_mut() {
            egui_renderer.free_textures_after_submit();
        }

        finish_result
    }

    pub fn resize_surface(
        &self,
        surface_frame_controller: &mut SurfaceFrameController,
        size: Size,
    ) -> Result<()> {
        let mut inner = self.inner.write().expect("SceneRenderer lock poisoned");
        surface_frame_controller.resize(&mut inner.ctx, size)
    }

    pub fn viewport_size(&self) -> Size {
        self.inner
            .read()
            .expect("SceneRenderer lock poisoned")
            .viewport_size()
    }

    pub fn set_light(&self, light: LightSource) -> Result<()> {
        self.inner
            .write()
            .expect("SceneRenderer lock poisoned")
            .set_light(light)
    }

    pub fn set_outline_color(&self, color: Vec4) {
        self.inner
            .write()
            .expect("SceneRenderer lock poisoned")
            .set_outline_color(color);
    }

    pub fn set_outline(&self, color: Vec4, thickness: f32) {
        self.inner
            .write()
            .expect("SceneRenderer lock poisoned")
            .set_outline(color, thickness);
    }

    pub fn get_device(&self) -> Arc<Device> {
        self.inner
            .read()
            .expect("SceneRenderer lock poisoned")
            .ctx
            .device
            .clone()
    }

    pub fn surface_format(&self) -> Result<TextureFormat> {
        self.inner
            .read()
            .expect("SceneRenderer lock poisoned")
            .ctx
            .surface_configuration
            .as_ref()
            .map(|configuration| configuration.format)
            .ok_or_else(|| anyhow::anyhow!("renderer surface is not configured"))
    }
}

impl SceneRendererInner {
    fn update_gpu(&mut self, delta_time: DeltaTime64, camera: &Camera) {
        self.animators.values_mut().for_each(|animator| {
            if let Err(animator_error) = animator.play(delta_time) {
                error!("{:?}", animator_error)
            }
        });

        self.gpu_resources.update(&self.ctx.queue, camera);
    }

    fn viewport_size(&self) -> Size {
        self.ctx.size
    }

    fn set_outline_color(&mut self, color: Vec4) {
        self.gpu_resources.set_outline_color(&self.ctx.queue, color);
    }

    fn set_outline(&mut self, color: Vec4, thickness: f32) {
        self.gpu_resources.set_outline(&self.ctx, color, thickness);
    }

    fn set_light(&mut self, light: LightSource) -> Result<()> {
        self.gpu_resources.set_light(&self.ctx, light)
    }
}
