use std::ops::Deref;

use wgpu::{
    Buffer, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    Shared,
    types::{
        BaseBuffer, TransformBuffer,
        ids::{UniformBufferId, UniformResourceId},
        transform::Transform,
    },
};

use super::Id;

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct UniformBuffer {
    id: UniformBufferId,
    buffer: Buffer,
    transform: Shared<Transform>,
}

impl Deref for UniformBuffer {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl UniformBuffer {
    pub fn new(
        id: UniformBufferId,
        device: &Device,
        contents: &[u8],
        transform: Shared<Transform>,
    ) -> Self {
        Self {
            id: id.clone(),
            buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some(id.get()),
                contents,
                usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            }),
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
    fn get_transform(&self) -> Shared<Transform> {
        self.transform.clone()
    }
}
