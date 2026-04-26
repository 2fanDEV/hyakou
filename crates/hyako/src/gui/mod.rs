use std::{ops::Deref, sync::Arc};

use egui::{ClippedPrimitive, Context, Id, RawInput, ViewportId};
use egui_wgpu::{RendererOptions, ScreenDescriptor};
use egui_winit::State;
use log::debug;
use wgpu::{
    CommandBuffer, Device, Queue, RenderPassDescriptor, TextureFormat, rwh::HasDisplayHandle,
    wgt::CommandEncoderDescriptor,
};
use winit::window::{Theme, Window};

pub struct GeneratedPanelOutput {
    pub clipped_primitives: Vec<ClippedPrimitive>,
    pub pixels_per_point: f32,
}

pub trait EguiPanel {
    fn generate(&mut self, context: &Context, raw_input: &RawInput) -> GeneratedPanelOutput;
}

pub struct CameraPanel {
    open: bool,
    speed: f32,
}

impl CameraPanel {
    pub fn new(camera_speed: f32) -> Self {
        Self {
            open: true,
            speed: camera_speed,
        }
    }
}

impl EguiPanel for CameraPanel {
    fn generate(&mut self, context: &Context, raw_input: &RawInput) -> GeneratedPanelOutput {
        let output = context.run_ui(raw_input.deref().clone(), |ui| {
            ui.label("Camera");
            if ui.button("Translate").clicked() {
                debug!("Translation begun!")
            }
        });
        GeneratedPanelOutput {
            clipped_primitives: context.tessellate(output.shapes, output.pixels_per_point),
            pixels_per_point: output.pixels_per_point,
        }
    }
}

pub struct EguiRenderer {
    state: State,
    context: Context,
    device: Arc<Device>,
    window: Arc<Window>,
    renderer: egui_wgpu::Renderer,
}

impl EguiRenderer {
    pub fn new(
        device: Arc<Device>,
        window: Arc<Window>,
        format: TextureFormat,
        opt: RendererOptions,
    ) -> Self {
        Self {
            state: State::new(
                Context::default(),
                ViewportId(Id::new("Egui Renderer")),
                &window.display_handle().unwrap(),
                Some(window.scale_factor() as f32),
                Some(Theme::Dark),
                None,
            ),
            context: Context::default(),
            device: device.clone(),
            window,
            renderer: egui_wgpu::Renderer::new(&device, TextureFormat::Rgba8Unorm, opt),
        }
    }

    pub fn render(&mut self, queue: &Queue, ui: &mut dyn EguiPanel) {
        let egui_input = self.state.egui_input();
        let mut command_encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Egui Encoder"),
            });
        {
            let primitives = ui.generate(&self.context, egui_input);

            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [800, 400],
                pixels_per_point: primitives.pixels_per_point,
            };

            self.renderer.update_buffers(
                &self.device,
                queue,
                &mut command_encoder,
                &primitives.clipped_primitives,
                &screen_descriptor,
            );

            {
                let mut render_pass = command_encoder
                    .begin_render_pass(&RenderPassDescriptor {
                        label: (),
                        color_attachments: (),
                        depth_stencil_attachment: (),
                        timestamp_writes: (),
                        occlusion_query_set: (),
                        multiview_mask: (),
                    })
                    .forget_lifetime();

                self.renderer.render(
                    &mut render_pass,
                    &primitives.clipped_primitives,
                    &screen_descriptor,
                );
            } // render_pass drops here, encoder unlocks

            queue.submit(std::iter::once(command_encoder.finish()));
        }
    }
}
