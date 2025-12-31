use parking_lot::RwLock;
use std::{ops::Deref, sync::Arc};

use anyhow::Result;
use wgpu::{Buffer, Queue};

use crate::renderer::components::transform::Transform;

pub mod ids;
pub mod uniform;

pub type DeltaTime = f32;

pub trait Id {
    fn get_id(&self) -> &str;
}

trait BaseBuffer {
    fn get_buffer(&self) -> &Buffer;
    fn get_id_cloned(&self) -> Box<dyn Id>;
    fn get_id_as_string(&self) -> &str;
}

pub trait TransformBuffer: Deref + BaseBuffer {
    fn get_transform(&self) -> Arc<RwLock<Transform>>;
    fn update_buffer_transform(&mut self, queue: &Queue, data: &[u8]) -> Result<()> {
        let buffer = self.get_buffer();
        queue.write_buffer(buffer, 0, data);
        Ok(())
    }
}
