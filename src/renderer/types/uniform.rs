use std::{ops::Deref, sync::Arc};

use bytemuck::bytes_of;
use wgpu::Buffer;

use crate::renderer::{
    components::transform::Transform,
    types::{
        BaseBuffer, TransformBuffer,
        ids::{UniformBufferId, UniformResourceId},
    },
};

use super::Id;

#[derive(Clone)]
pub struct UniformBuffer {
    id: UniformBufferId,
    buffer: Buffer,
    transform: Arc<Transform>,
}

impl Deref for UniformBuffer {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl UniformBuffer {
    pub fn new(id: UniformBufferId, buffer: Buffer, transform: Arc<Transform>) -> Self {
        Self {
            id,
            buffer,
            transform,
        }
    }
}

impl BaseBuffer for UniformBuffer {
    fn get_buffer(&self) -> &Buffer {
        &self.deref()
    }
    fn get_id_as_string(&self) -> &str {
        self.id.get()
    }

    fn get_id_cloned(&self) -> Box<dyn Id> {
        Box::new(self.id.clone())
    }
}

impl TransformBuffer for UniformBuffer {
    fn get_transform(&self) -> Arc<Transform> {
        self.transform.clone()
    }

    fn update_buffer_transform(&mut self, queue: &wgpu::Queue) -> anyhow::Result<()> {
        let matrix = self.transform.get_matrix();
        queue.write_buffer(&self.buffer, 0, bytes_of(&matrix));
        Ok(())
    }
}
