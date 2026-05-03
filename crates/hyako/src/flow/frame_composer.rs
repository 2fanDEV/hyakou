use crate::{
    gui::{EguiRenderer, panels::camera_panel::CameraPanel},
    renderer::frame::FrameTarget,
};

pub struct FrameComposer {
    camera_panel: CameraPanel,
}

impl FrameComposer {
    pub fn new() -> Self {
        Self {
            camera_panel: CameraPanel::new(2.0),
        }
    }

    pub fn compose_frame(
        &mut self,
        target: &mut FrameTarget<'_>,
        mut egui_renderer: Option<&mut EguiRenderer>,
    ) {
        if let Some(egui_renderer) = egui_renderer.as_mut() {
            egui_renderer.render(target, |ui| {
                self.camera_panel.show(ui.ctx());
            });
        }
    }
}

impl Default for FrameComposer {
    fn default() -> Self {
        Self::new()
    }
}
