use glam::{Mat4, Vec3};

use crate::{
    animations::trajectory::calculate_direction_vector,
    types::{
        Size,
        base::Id,
        camera::{Pitch, Yaw},
    },
};

#[derive(Debug, Default)]
pub struct Camera {
    pub id: Id,
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub yaw: Yaw,
    pub pitch: Pitch,
    pub speed: f32,
    pub sensitivity: f32,
    pub smoothing_factor: f32,
    pub precalculated_smoothing: f32,
}

impl Camera {
    const DEFAULT_ASPECT_RATIO: f32 = 1.0;

    pub fn aspect_ratio_from_size(size: Size) -> f32 {
        if size.height == 0 {
            Self::DEFAULT_ASPECT_RATIO
        } else {
            size.width as f32 / size.height as f32
        }
    }

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
            id: Id::uuid(),
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
            yaw,
            pitch,
            speed,
            sensitivity,
            smoothing_factor,
            precalculated_smoothing: 1.0 - smoothing_factor,
        }
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = if aspect.is_finite() && aspect > 0.0 {
            aspect
        } else {
            Self::DEFAULT_ASPECT_RATIO
        };
    }

    pub fn set_aspect_from_size(&mut self, size: Size) {
        self.set_aspect(Self::aspect_ratio_from_size(size));
    }

    pub fn update_yaw(&mut self, yaw_delta: f32) {
        self.yaw.update(yaw_delta);
    }

    pub fn update_pitch(&mut self, pitch_delta: f32) {
        self.pitch.update(pitch_delta);
    }

    pub fn move_camera(&mut self, yaw_delta: f32, pitch_delta: f32) {
        self.yaw.add(
            yaw_delta * self.sensitivity,
            self.precalculated_smoothing,
            self.smoothing_factor,
        );
        self.pitch.add(
            pitch_delta * self.sensitivity,
            self.precalculated_smoothing,
            self.smoothing_factor,
        );
        let forward = calculate_direction_vector(*self.yaw, *self.pitch);
        self.target = self.eye + forward;
    }

    pub fn build_view_proj_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
        proj * view
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::{
        components::camera::camera::Camera,
        types::{
            Size,
            camera::{Pitch, Yaw},
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

    #[test]
    fn test_delta_to_rotation_conversion_yaw() {
        let mut camera = create_test_camera();
        let initial_yaw = camera.yaw;

        camera.move_camera(1.0, 0.0);

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

        camera.move_camera(0.0, 10.0);

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

        camera.move_camera(-10.0, -10.0);

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
    fn test_sensitivity_scaling_double_sensitivity() {
        let mut camera_low = create_test_camera_sensitivity(0.5);
        let mut camera_high = create_test_camera_sensitivity(1.0);
        let yaw_delta = 10.0;
        let pitch_delta = 10.0;

        // Apply same delta to both controllers
        camera_low.move_camera(yaw_delta, pitch_delta);
        camera_high.move_camera(yaw_delta, pitch_delta);

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

        camera.move_camera(100.0, 100.0);

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

        for _ in 0..100 {
            camera.move_camera(0.0, 10.0);
        }

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

        for _ in 0..100 {
            camera.move_camera(0.0, -10.0);
        }

        let min_pitch = -89.0_f32.to_radians();
        assert!(
            *camera.pitch >= min_pitch,
            "Pitch should be clamped at lower limit. Got: {}, Min: {}",
            *camera.pitch,
            min_pitch
        );
    }

    #[test]
    fn test_set_aspect_updates_projection_matrix() {
        let mut camera = create_test_camera();
        let initial_matrix = camera.build_view_proj_matrix().to_cols_array();

        camera.set_aspect(4.0 / 3.0);

        assert_ne!(
            initial_matrix,
            camera.build_view_proj_matrix().to_cols_array(),
            "Projection matrix should change when the aspect ratio changes"
        );
    }

    #[test]
    fn test_set_aspect_rejects_invalid_values() {
        let mut camera = create_test_camera();

        camera.set_aspect(0.0);
        assert_eq!(camera.aspect, 1.0);

        camera.set_aspect(f32::NAN);
        assert_eq!(camera.aspect, 1.0);
    }

    #[test]
    fn test_aspect_ratio_from_size_defaults_for_zero_height() {
        assert_eq!(
            Camera::aspect_ratio_from_size(Size {
                width: 1920,
                height: 0,
            }),
            1.0
        );
    }
}
