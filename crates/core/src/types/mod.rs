use std::ops::Deref;

use anyhow::Result;
use wgpu::{Buffer, Queue};

use crate::{Shared, types::transform::Transform};

pub mod base;
pub mod camera;
pub mod ids;
pub mod mouse_delta;
pub mod shared;
pub mod transform;
pub mod upload_status;

pub type DeltaTime = f32;
pub type DeltaTime64 = f64;

pub const F32_ZERO: f32 = 0.0;
pub const F64_ZERO: f64 = 0.0;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    const MIN_GPU_DIMENSION: u32 = 1;

    pub fn is_zero(self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub fn clamp_size_for_gpu(self) -> Self {
        Self {
            width: self.width.max(Self::MIN_GPU_DIMENSION),
            height: self.height.max(Self::MIN_GPU_DIMENSION),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ModelMatrixBindingMode {
    Immediate,
    Uniform,
}

pub trait BaseId {
    fn get_id(&self) -> &str;
}

#[allow(unused)]
pub trait BaseBuffer {
    fn get_buffer(&self) -> &Buffer;
    fn get_id_cloned(&self) -> Box<dyn BaseId>;
    fn get_id_as_string(&self) -> &str;
}

pub trait TransformBuffer: Deref + BaseBuffer {
    fn get_transform(&self) -> Shared<Transform>;
    fn update_buffer_transform(&mut self, queue: &Queue, data: &[u8]) -> Result<()> {
        let buffer = self.get_buffer();
        queue.write_buffer(buffer, 0, data);
        Ok(())
    }
}
