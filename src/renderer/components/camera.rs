use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Buffer, BufferBinding, Device, ShaderStages,
};
use winit::keyboard::KeyCode;

use crate::renderer::geometry::BindGroupProvider;

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(camera_speed: f32) -> CameraController {
        Self {
            speed: camera_speed,
            is_backward_pressed: false,
            is_forward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn handle_key(&mut self, key_code: KeyCode, is_pressed: bool) -> bool {
        match key_code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, delta_time: f32) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.length();
        let speed = self.speed * delta_time;

        if self.is_forward_pressed && forward_mag > speed {
            camera.eye += forward_norm * speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * speed;
        }

        let right = forward_norm.cross(camera.up);
        if self.is_right_pressed {
            camera.eye = camera.target - (forward + right * speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * speed).normalize() * forward_mag;
        }
    }
}

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
        self.view_projection_matrix = camera.build_proj_matrix();
    }

    pub fn bind_group(device: &Device, buffer: &Buffer) -> (BindGroupLayout, BindGroup) {
        let layout = Self::bind_group_layout(device);
        (
            layout.clone(),
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("Camera Bind Group"),
                layout: &layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
            }),
        )
    }
}

impl BindGroupProvider for CameraUniform {
    fn bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera Buffer"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }
}

pub struct Camera {
    eye: Vec3,
    target: Vec3,
    up: Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn new(
        eye: Vec3,
        target: Vec3,
        up: Vec3,
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

    pub fn build_proj_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_rh(self.aspect, self.fovy, self.znear, self.zfar);
        proj * view
    }
}
