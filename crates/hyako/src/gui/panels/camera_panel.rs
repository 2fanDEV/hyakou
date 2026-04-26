use egui::{Context, RawInput};
use log::debug;

use crate::gui::panels::{EguiPanel, GeneratedPanelOutput};

pub struct CameraPanel {
    open: bool,
    speed: f32,
    is_rendered: bool,
}

impl CameraPanel {
    pub fn new(camera_speed: f32) -> Self {
        Self {
            open: true,
            speed: camera_speed,
            is_rendered: {
                #[cfg(target_arch = "wasm32")]
                {
                    false
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    true
                }
            },
        }
    }
}

impl EguiPanel for CameraPanel {
    fn generate(&mut self, context: &Context, raw_input: RawInput) -> GeneratedPanelOutput {
        let output = context.run_ui(raw_input, |ui| {
            egui::Window::new("Camera")
                .open(&mut self.open)
                .show(ui.ctx(), |ui| {
                    ui.label("Camera");
                    ui.add(egui::Slider::new(&mut self.speed, 0.0..=100.0).text("Speed"));
                    if ui.button("Translate").clicked() {
                        debug!("Translation begun!")
                    }
                });
        });
        GeneratedPanelOutput {
            clipped_primitives: context.tessellate(output.shapes, output.pixels_per_point),
            pixels_per_point: output.pixels_per_point,
            textures_delta: output.textures_delta,
            platform_output: output.platform_output,
        }
    }

    fn should_be_rendered(&self) -> bool {
        self.is_rendered
    }

    fn rendered(&mut self, render: bool) {
        self.is_rendered = render;
    }
}
