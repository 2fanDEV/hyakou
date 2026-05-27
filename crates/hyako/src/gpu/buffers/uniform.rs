use std::ops::Deref;

use hyakou_core::types::ids::UniformBufferId;
use wgpu::{
    Buffer, BufferUsages, Device, Queue,
    util::{BufferInitDescriptor, DeviceExt},
};

#[derive(Debug, Clone)]
pub struct UniformBuffer {
    buffer: Buffer,
}

impl Deref for UniformBuffer {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl UniformBuffer {
    pub fn new(id: UniformBufferId, device: &Device, contents: &[u8]) -> Self {
        Self {
            buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some(id.as_str()),
                contents,
                usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            }),
        }
    }

    pub fn update_buffer_transform(&mut self, queue: &Queue, data: &[u8]) {
        queue.write_buffer(&self.buffer, 0, data);
    }
}
