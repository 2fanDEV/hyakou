use egui::Context;
use log::debug;

use crate::gui::widgets::text_editor::TextEditor;

pub struct CameraPanel {
    open: bool,
    speed: f32,
    is_rendered: bool,
    text_editor: TextEditor,
    read_only_text_editor: TextEditor,
}

impl CameraPanel {
    pub fn new(camera_speed: f32) -> Self {
        let mut text_editor = TextEditor::new("camera_panel_text_editor", "Camera note");
        text_editor.set_multiline(true);

        let mut read_only_text_editor = TextEditor::new(
            "camera_panel_read_only_text_editor",
            "Read-only camera note\nSecond line",
        );
        read_only_text_editor.set_multiline(true);
        read_only_text_editor.set_read_only(true);

        Self {
            open: true,
            speed: camera_speed,
            text_editor,
            read_only_text_editor,
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
                self.text_editor.show(ui);
                self.read_only_text_editor.show(ui);
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
