use std::{
    ops::Deref,
    sync::Arc,
};
use parking_lot::RwLock;

use wgpu::{
    Buffer, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

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
    transform: Arc<RwLock<Transform>>,
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
        transform: Arc<RwLock<Transform>>,
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
    fn get_transform(&self) -> Arc<RwLock<Transform>> {
        self.transform.clone()
    }
}
