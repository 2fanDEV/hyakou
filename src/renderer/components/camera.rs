use bytemuck::{Pod, Zeroable};
use nalgebra::{Matrix4, Point3, Vector3};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_projection_matrix: Matrix4<f32>
}

impl CameraUniform {
    pub fn new() -> CameraUniform {
        Self { 
            view_projection_matrix: Matrix4::identity()
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.view_projection_matrix = camera.build_proj_matrix();
    }
}

pub struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn new(
        eye: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn build_proj_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(&self.eye, &self.target, &self.up);
        let proj =
            Matrix4::new_perspective(self.aspect, self.fovy, self.znear, self.zfar);
        proj * view
    }
}
