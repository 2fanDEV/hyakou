use std::ops::Deref;

use egui::{ClippedPrimitive, Context, RawInput, Ui};
use egui_wgpu::RendererOptions;
use egui_winit::State;
use log::debug;
use wgpu::TextureFormat;

pub struct EguiInterface {
    state: State,
    context: Context,
    ui: Ui,
}

impl Deref for EguiInterface {
    type Target = Ui;

    fn deref(&self) -> &Self::Target {
        &self.ui
    }
}

impl EguiInterface {
    pub fn new<F>(state: State, ui: F) -> Self
    where
        F: FnOnce() -> egui::Ui,
    {
        Self {
            state,
            context: egui::Context::default(),
            ui: ui(),
        }
    }

    pub fn generate(&self, raw_input: RawInput) -> Vec<ClippedPrimitive> {
        let run_ui = self.ui.run_ui(raw_input, |ui| {});
        self.ui.tessellate(run_ui.shapes, run_ui.pixels_per_point)
    }
}

pub struct EguiRenderer {
    renderer: egui_wgpu::Renderer,
}

impl EguiRenderer {
    pub fn new(device: &wgpu::Device, format: TextureFormat, opt: RendererOptions) -> Self {
        Self {
            renderer: egui_wgpu::Renderer::new(device, format, opt),
        }
    }

    pub fn render(
        &mut self,
        ui: &mut EguiInterface,
        raw_input: RawInput,
        render_pass: &mut wgpu::RenderPass<'static>,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
    ) {
        let primitives = ui.generate(raw_input);
        self.renderer
            .render(render_pass, &primitives, screen_descriptor);
    }
}

fn main() {
    let context = egui::Context::default();
    let inner_response = egui::Window::new("Camera")
        .show(&context, |ui| {
            ui.label("Camera");
            if ui.button("Click").clicked() {
                debug!("Clicked!");
            }
        })
        .unwrap();
}
