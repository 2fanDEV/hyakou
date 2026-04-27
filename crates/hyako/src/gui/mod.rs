use std::sync::Arc;

use egui::{Context, TextureId, Ui, ViewportId};
use egui_wgpu::{RendererOptions, ScreenDescriptor};
use egui_winit::State;
use wgpu::{Device, RenderPassColorAttachment, RenderPassDescriptor, TextureFormat};
use winit::window::Window;

use crate::renderer::frame::FrameTarget;

pub mod panels;
pub mod primitives;
mod render_pass;

pub struct EguiRenderer {
    state: State,
    context: Context,
    device: Arc<Device>,
    window: Arc<Window>,
    renderer: egui_wgpu::Renderer,
    textures_to_free: Vec<TextureId>,
}

impl EguiRenderer {
    pub fn new(
        device: Arc<Device>,
        window: Arc<Window>,
        format: TextureFormat,
        opt: RendererOptions,
    ) -> Self {
        let context = Context::default();
        Self {
            state: State::new(
                context.clone(),
                ViewportId::ROOT,
                window.as_ref(),
                Some(window.scale_factor() as f32),
                window.theme(),
                None,
            ),
            context,
            device: device.clone(),
            window,
            renderer: egui_wgpu::Renderer::new(&device, format, opt),
            textures_to_free: Vec::new(),
        }
    }

    pub fn handle_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.state.on_window_event(&self.window, event).consumed
    }

    pub fn render(&mut self, target: &mut FrameTarget<'_>, mut render_ui: impl FnMut(&mut Ui)) {
        let egui_input = self.state.take_egui_input(&self.window);
        let output = self.context.run_ui(egui_input, |ui| render_ui(ui));
        self.state
            .handle_platform_output(&self.window, output.platform_output);
        let pixels_per_point = output.pixels_per_point;
        let clipped_primitives = self.context.tessellate(output.shapes, pixels_per_point);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: target.size_in_pixels,
            pixels_per_point,
        };

        for (id, image_delta) in &output.textures_delta.set {
            self.renderer
                .update_texture(&self.device, target.queue, *id, image_delta);
        }
        self.textures_to_free
            .extend(output.textures_delta.free.iter().cloned());

        self.renderer.update_buffers(
            &self.device,
            target.queue,
            target.encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        {
            let color_attachments = [Some(RenderPassColorAttachment {
                view: target.color_view,
                depth_slice: None,
                resolve_target: None,
                ops: render_pass::color_attachment_operations(),
            })];
            let render_pass_descriptor = RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: render_pass::depth_stencil_attachment(),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            };
            let mut render_pass = target
                .encoder
                .begin_render_pass(&render_pass_descriptor)
                .forget_lifetime();

            self.renderer
                .render(&mut render_pass, &clipped_primitives, &screen_descriptor);
        }
    }

    pub fn free_textures_after_submit(&mut self) {
        for id in self.textures_to_free.drain(..) {
            self.renderer.free_texture(&id);
        }
    }
}
