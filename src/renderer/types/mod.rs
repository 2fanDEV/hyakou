use std::{ops::Deref, sync::Arc};

use anyhow::Result;
use wgpu::{Buffer, Queue};

use crate::renderer::components::transform::Transform;

pub mod ids;
pub mod uniform;

trait Id {
    fn get_id(&self) -> &str;
}

trait BaseBuffer {
    fn get_buffer(&self) -> &Buffer;
    fn get_id_cloned(&self) -> Box<dyn Id>;
    fn get_id_as_string(&self) -> &str;
}

trait TransformBuffer: Deref + BaseBuffer {
    fn get_transform(&self) -> Arc<Transform>;
    fn update_buffer_transform(&mut self, queue: &Queue) -> Result<()> {
        let buffer = self.get_buffer();
        let current_transform = self.get_transform();

        Ok(())
    }
}
