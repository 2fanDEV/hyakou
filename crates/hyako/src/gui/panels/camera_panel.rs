use egui::Context;
use log::debug;

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

impl CameraPanel {
    pub fn show(&mut self, context: &Context) {
        if !self.is_rendered {
            return;
        }

        egui::Window::new("Camera")
            .open(&mut self.open)
            .show(context, |ui| {
                ui.label("Camera");
                ui.add(egui::Slider::new(&mut self.speed, 0.0..=100.0).text("Speed"));
                if ui.button("Translate").clicked() {
                    debug!("Translation begun!")
                }
            });
    }

    pub fn should_be_rendered(&self) -> bool {
        self.is_rendered
    }

    pub fn rendered(&mut self, render: bool) {
        self.is_rendered = render;
    }
}
