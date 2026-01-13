use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Buffer, BufferBinding, Device, ShaderStages,
};

use crate::renderer::{
    animator::trajectory::calculate_direction_vector,
    geometry::BindGroupProvider,
    types::{
        camera::{Pitch, Yaw},
        mouse_delta::{MouseAction, MouseButton, MouseDelta},
    },
};

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

    fn bind_group(
        device: &Device,
        buffer: &Buffer,
        bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        })
    }
}

#[derive(Debug, Default)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub yaw: Yaw,
    pub pitch: Pitch,
    pub camera_speed: f32,
    pub sensitivity: f32,
    pub smoothing_factor: f32,
    pub precalculated_smoothing: f32,
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
        yaw: Yaw,
        pitch: Pitch,
        speed: f32,
        sensitivity: f32,
        smoothing_factor: f32,
    ) -> Self {
        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
            yaw,
            pitch,
            camera_speed: speed,
            sensitivity,
            smoothing_factor,
            precalculated_smoothing: 1.0 - smoothing_factor,
        }
    }

    pub fn move_camera_with_mouse(&mut self, mouse_delta: &MouseDelta) {
        if mouse_delta.state.get_action().eq(&MouseAction::Clicked)
            && mouse_delta.state.get_button().eq(&MouseButton::Left)
            && mouse_delta.is_mouse_on_window()
        {
            self.yaw.add(
                mouse_delta.delta_position.x() as f32 * self.sensitivity,
                self.precalculated_smoothing,
                self.smoothing_factor,
            );
            self.pitch.add(
                mouse_delta.delta_position.y() as f32 * self.sensitivity,
                self.precalculated_smoothing,
                self.smoothing_factor,
            );
            let forward = calculate_direction_vector(*self.yaw, *self.pitch);
            self.target = self.eye + forward;
        }
    }

    pub fn build_proj_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
        proj * view
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::renderer::{
        components::camera::Camera,
        types::{
            camera::{Pitch, Yaw},
            mouse_delta::{
                MouseAction, MouseButton, MouseDelta, MousePosition, MouseState, MovementDelta,
            },
        },
    };

    fn create_test_camera() -> Camera {
        Camera::new(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            16.0 / 9.0,
            45.0_f32.to_radians(),
            0.1,
            100.0,
            Yaw::new(0.0),
            Pitch::new(0.0),
            20.0,
            0.5,
            0.5,
        )
    }

    fn create_test_camera_sensitivity(sensitivity: f32) -> Camera {
        Camera::new(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            16.0 / 9.0,
            45.0_f32.to_radians(),
            0.1,
            100.0,
            Yaw::new(0.0),
            Pitch::new(0.0),
            20.0,
            sensitivity,
            0.5,
        )
    }

    // Helper function to create MouseDelta for testing
    fn create_mouse_delta(delta_x: f64, delta_y: f64, is_clicked: bool) -> MouseDelta {
        MouseDelta {
            delta_position: MovementDelta::new(delta_x, delta_y),
            state: MouseState::new(
                MouseButton::Left,
                if is_clicked {
                    MouseAction::Clicked
                } else {
                    MouseAction::Released
                },
            ),
            is_mouse_on_window: true,
            position: MousePosition::new(0.0, 0.0),
        }
    }

    #[test]
    fn test_delta_to_rotation_conversion_yaw() {
        let mut camera = create_test_camera();
        let initial_yaw = camera.yaw;

        let mouse_delta = create_mouse_delta(10.0, 0.0, true);
        camera.move_camera_with_mouse(&mouse_delta);

        // Yaw should have changed due to positive X delta
        // Note: Due to smoothing, the change won't be exactly 10.0
        // First movement with smoothing_factor 0.5: smoothed = 0.0 * 0.5 + 10.0 * 0.5 = 5.0
        assert!(
            *camera.yaw > *initial_yaw,
            "Yaw should increase with positive X delta"
        );
    }

    #[test]
    fn test_delta_to_rotation_conversion_pitch() {
        let mut camera = create_test_camera();
        let initial_pitch = camera.pitch;

        let mouse_delta = create_mouse_delta(0.0, 10.0, true);
        camera.move_camera_with_mouse(&mouse_delta);

        // Pitch should decrease due to positive Y delta (inverted Y-axis)
        // First movement with smoothing_factor 0.5: smoothed = 0.0 * 0.5 + 10.0 * 0.5 = 5.0
        assert!(
            *camera.pitch < *initial_pitch,
            "Pitch should decrease with positive Y delta (inverted Y-axis)"
        );
    }

    #[test]
    fn test_delta_to_rotation_negative_values() {
        let mut camera = create_test_camera();
        let initial_yaw = camera.yaw;
        let initial_pitch = camera.pitch;

        let mouse_delta = create_mouse_delta(-10.0, -10.0, true);
        camera.move_camera_with_mouse(&mouse_delta);

        // Negative X delta should decrease yaw
        assert!(
            *camera.yaw < *initial_yaw,
            "Yaw should decrease with negative X delta"
        );
        // Negative Y delta should increase pitch (inverted Y-axis: subtract negative = add)
        assert!(
            *camera.pitch > *initial_pitch,
            "Pitch should increase with negative Y delta (inverted Y-axis)"
        );
    }

    #[test]
    fn test_rotation_only_when_mouse_clicked() {
        let mut camera = create_test_camera();
        let initial_yaw = camera.yaw;
        let initial_pitch = camera.pitch;

        // Mouse delta with button released should not rotate
        let mouse_delta = create_mouse_delta(10.0, 10.0, false);
        camera.move_camera_with_mouse(&mouse_delta);

        assert_eq!(
            *camera.yaw, *initial_yaw,
            "Yaw should not change when mouse button is released"
        );
        assert_eq!(
            *camera.pitch, *initial_pitch,
            "Pitch should not change when mouse button is released"
        );
    }

    #[test]
    fn test_sensitivity_scaling_double_sensitivity() {
        let mut camera_low = create_test_camera_sensitivity(0.5);
        let mut camera_high = create_test_camera_sensitivity(1.0);

        let mouse_delta = create_mouse_delta(10.0, 10.0, true);

        // Apply same delta to both controllers
        camera_low.move_camera_with_mouse(&mouse_delta);
        camera_high.move_camera_with_mouse(&mouse_delta);

        // Controller with 2x sensitivity should rotate more (accounting for smoothing)
        let yaw_change_low = *camera_low.yaw;
        let yaw_change_high = *camera_high.yaw;

        assert!(
            yaw_change_high > yaw_change_low,
            "Higher sensitivity should produce larger rotation. Low: {}, High: {}",
            yaw_change_low,
            yaw_change_high
        );

        // The ratio should be close to 2.0 (accounting for smoothing factor)
        let ratio = yaw_change_high / yaw_change_low;
        assert!(
            (ratio - 2.0).abs() < 0.1,
            "Sensitivity scaling should be approximately 2x, got ratio: {}",
            ratio
        );
    }

    #[test]
    fn test_sensitivity_zero_produces_no_rotation() {
        let mut camera = create_test_camera_sensitivity(0.0);
        let initial_yaw = *camera.yaw;
        let initial_pitch = *camera.pitch;

        let mouse_delta = create_mouse_delta(100.0, 100.0, true);
        camera.move_camera_with_mouse(&mouse_delta);

        // With zero sensitivity, no rotation should occur even with large delta
        assert_eq!(
            *camera.yaw, initial_yaw,
            "Yaw should not change with zero sensitivity"
        );
        assert_eq!(
            *camera.pitch, initial_pitch,
            "Pitch should not change with zero sensitivity"
        );
    }

    #[test]
    fn test_pitch_clamping_at_upper_limit() {
        let mut camera = create_test_camera();

        // Apply large upward rotation to exceed pitch limit
        for _ in 0..100 {
            let mouse_delta = create_mouse_delta(0.0, 10.0, true);
            camera.move_camera_with_mouse(&mouse_delta);
        }

        // Pitch should be clamped to max (89 degrees)
        let max_pitch = 89.0_f32.to_radians();
        assert!(
            *camera.pitch <= max_pitch,
            "Pitch should be clamped at upper limit. Got: {}, Max: {}",
            *camera.pitch,
            max_pitch
        );
    }

    #[test]
    fn test_pitch_clamping_at_lower_limit() {
        let mut camera = create_test_camera();

        // Apply large downward rotation to exceed pitch limit
        for _ in 0..100 {
            let mouse_delta = create_mouse_delta(0.0, -10.0, true);
            camera.move_camera_with_mouse(&mouse_delta);
        }

        // Pitch should be clamped to min (-89 degrees)
        let min_pitch = -89.0_f32.to_radians();
        assert!(
            *camera.pitch >= min_pitch,
            "Pitch should be clamped at lower limit. Got: {}, Min: {}",
            *camera.pitch,
            min_pitch
        );
    }
}
