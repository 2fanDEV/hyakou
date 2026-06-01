use crate::gpu::uniform::GpuUniformMetadata;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use hyakou_core::components::camera::camera::Camera;
use wgpu::ShaderStages;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_projection_matrix: Mat4,
}

impl CameraUniform {
    pub fn new() -> CameraUniform {
        Self {
            view_projection_matrix: Mat4::IDENTITY,
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.view_projection_matrix = camera.build_view_proj_matrix();
    }
}

impl GpuUniformMetadata for CameraUniform {
    const LAYOUT_LABEL: &'static str = "Camera Buffer";
    const BIND_GROUP_LABEL: &'static str = "Camera Bind Group";
    const VISIBILITY: ShaderStages = ShaderStages::VERTEX;
}
