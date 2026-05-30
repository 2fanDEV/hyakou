pub mod camera;
pub mod ids;
pub mod import_diagnostic;
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
// NOTE: This is render-policy state in core for now. Move it into hyako when the
// render boundary is finalized during the ECS migration.
pub enum ModelMatrixBindingMode {
    Immediate,
    Uniform,
}
