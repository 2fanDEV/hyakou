use egui::{ClippedPrimitive, Context, PlatformOutput, RawInput, TexturesDelta};

pub mod camera_panel;

pub struct GeneratedPanelOutput {
    pub clipped_primitives: Vec<ClippedPrimitive>,
    pub pixels_per_point: f32,
    pub textures_delta: TexturesDelta,
    pub platform_output: PlatformOutput,
}

pub trait EguiPanel {
    fn generate(&mut self, context: &Context, raw_input: RawInput) -> GeneratedPanelOutput;
    fn should_be_rendered(&self) -> bool;
    fn rendered(&mut self, render: bool);
}
