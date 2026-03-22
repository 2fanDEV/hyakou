use std::sync::mpsc::Receiver;

use crate::renderer::Renderer;

pub mod renderer;
pub mod state;

pub struct RendererCommands {
   RESUMED
}


pub struct Orchestrator {
    pub renderer: Renderer,
    pub rx: Receiver<Renderer>,
}
