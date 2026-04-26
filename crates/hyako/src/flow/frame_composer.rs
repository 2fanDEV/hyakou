use anyhow::Result;

use crate::{
    gui::{EguiRenderer, panels::camera_panel::CameraPanel},
    renderer::Renderer,
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

    pub fn render_frame(
        &mut self,
        renderer: &mut Renderer,
        mut egui_renderer: Option<&mut EguiRenderer>,
        dt: f64,
    ) -> Result<()> {
        renderer.update(dt);

        let Some(mut frame) = renderer.begin_frame()? else {
            return Ok(());
        };

        {
            let mut target = frame.target();
            renderer.render_scene(&mut target);

            if let Some(egui_renderer) = egui_renderer.as_mut() {
                egui_renderer.render(&mut target, &mut self.camera_panel);
            }
        }

        let finish_result = renderer.finish_frame(frame);

        if let Some(egui_renderer) = egui_renderer.as_mut() {
            egui_renderer.free_textures_after_submit();
        }

        finish_result
    }
}

impl Default for FrameComposer {
    fn default() -> Self {
        Self::new()
    }
}
