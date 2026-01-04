use std::f32::consts::PI;

use anyhow::{Ok, Result};
use winit::keyboard::KeyCode;

use crate::renderer::{
    components::camera::Camera,
    types::{
        F32_ZERO,
        camera::{Pitch, Yaw},
        mouse_delta::{MouseAction, MouseDelta},
    },
};

#[derive(Debug, Default)]
pub struct CameraController {
    speed: f32,
    camera: Camera,
    yaw: Yaw,
    pitch: Pitch,
    sensitivity: f32,
    smoothing_factor: f32,
    precalculated_smoothing: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    const SMOOTHING_FACTOR: f32 = 0.35;

    pub fn new(camera_speed: f32, sensitivity: f32, camera: Camera) -> CameraController {
        Self {
            speed: camera_speed,
            is_backward_pressed: false,
            is_forward_pressed: false,
            yaw: Yaw::new(-PI / 2.0),
            pitch: Pitch::new(F32_ZERO),
            precalculated_smoothing: 1.0 - Self::SMOOTHING_FACTOR,
            smoothing_factor: Self::SMOOTHING_FACTOR,
            sensitivity,
            camera,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn get_camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn rotate(&mut self, mouse_delta: &MouseDelta) -> Result<()> {
        if mouse_delta.is_mouse_on_window
            && mouse_delta.state.get_action().eq(&MouseAction::CLICKED)
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
            self.camera.move_camera_with_mouse(&self.yaw, &self.pitch);
        }
        Ok(())
    }

    pub fn update_smoothing_factor(&mut self, smoothing_factor: f32) {
        self.smoothing_factor = smoothing_factor;
        self.precalculated_smoothing = 1.0 - smoothing_factor;
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

    pub fn update_camera(&mut self, delta_time: f32) {
        let forward = self.camera.target - self.camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.length();
        let speed = self.speed * delta_time;
        if self.is_forward_pressed && forward_mag > speed {
            self.camera.eye += forward_norm * speed;
        }
        if self.is_backward_pressed {
            self.camera.eye -= forward_norm * speed;
        }

        let right = forward_norm.cross(self.camera.up);
        if self.is_right_pressed {
            self.camera.eye =
                self.camera.target - (forward + right * speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            self.camera.eye =
                self.camera.target - (forward - right * speed).normalize() * forward_mag;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::types::mouse_delta::{
        MouseAction, MouseButton, MouseDelta, MousePosition, MouseState, MovementDelta,
    };
    use glam::Vec3;

    #[test]
    fn test_smoothing_factor() {
        let mut camera_controller = CameraController::new(0.0, 0.0, create_test_camera());
        camera_controller.update_smoothing_factor(0.5);
        assert!(camera_controller.smoothing_factor.eq(&0.5));
        assert!(camera_controller.precalculated_smoothing.eq(&0.5))
    }

    fn create_test_camera() -> Camera {
        Camera::new(
            Vec3::new(0.0, 0.0, 10.0), // eye (increased distance to avoid boundary conditions)
            Vec3::new(0.0, 0.0, 0.0),  // target
            Vec3::new(0.0, 1.0, 0.0),  // up
            16.0 / 9.0,                // aspect
            45.0_f32.to_radians(),     // fovy
            0.1,                       // znear
            100.0,                     // zfar
        )
    }

    #[test]
    fn test_new_controller_has_correct_initial_state() {
        let camera = create_test_camera();
        let controller = CameraController::new(5.0, 0.5, camera);

        assert_eq!(controller.speed, 5.0);
        assert_eq!(controller.is_forward_pressed, false);
        assert_eq!(controller.is_backward_pressed, false);
        assert_eq!(controller.is_left_pressed, false);
        assert_eq!(controller.is_right_pressed, false);
    }

    #[test]
    fn test_handle_key_w_sets_forward_pressed() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 0.5, camera);

        let handled = controller.handle_key(KeyCode::KeyW, true);

        assert!(handled);
        assert!(controller.is_forward_pressed);
    }

    #[test]
    fn test_handle_key_arrow_up_sets_forward_pressed() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 0.5, camera);

        let handled = controller.handle_key(KeyCode::ArrowUp, true);

        assert!(handled);
        assert!(controller.is_forward_pressed);
    }

    #[test]
    fn test_handle_key_s_sets_backward_pressed() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 0.5, camera);

        let handled = controller.handle_key(KeyCode::KeyS, true);

        assert!(handled);
        assert!(controller.is_backward_pressed);
    }

    #[test]
    fn test_handle_key_arrow_down_sets_backward_pressed() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 0.5, camera);

        let handled = controller.handle_key(KeyCode::ArrowDown, true);

        assert!(handled);
        assert!(controller.is_backward_pressed);
    }

    #[test]
    fn test_handle_key_a_sets_left_pressed() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 0.5, camera);

        let handled = controller.handle_key(KeyCode::KeyA, true);

        assert!(handled);
        assert!(controller.is_left_pressed);
    }

    #[test]
    fn test_handle_key_arrow_left_sets_left_pressed() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 0.5, camera);

        let handled = controller.handle_key(KeyCode::ArrowLeft, true);

        assert!(handled);
        assert!(controller.is_left_pressed);
    }

    #[test]
    fn test_handle_key_d_sets_right_pressed() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 0.5, camera);

        let handled = controller.handle_key(KeyCode::KeyD, true);

        assert!(handled);
        assert!(controller.is_right_pressed);
    }

    #[test]
    fn test_handle_key_arrow_right_sets_right_pressed() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 0.5, camera);

        let handled = controller.handle_key(KeyCode::ArrowRight, true);

        assert!(handled);
        assert!(controller.is_right_pressed);
    }

    #[test]
    fn test_handle_key_release_clears_state() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 0.5, camera);

        controller.handle_key(KeyCode::KeyW, true);
        assert!(controller.is_forward_pressed);

        controller.handle_key(KeyCode::KeyW, false);
        assert!(!controller.is_forward_pressed);
    }

    #[test]
    fn test_handle_key_unhandled_key_returns_false() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 0.5, camera);

        let handled = controller.handle_key(KeyCode::Space, true);

        assert!(!handled);
    }

    #[test]
    fn test_update_camera_forward_movement() {
        let camera = create_test_camera();
        let initial_eye = camera.eye;
        let mut controller = CameraController::new(5.0, 0.5, camera);
        controller.is_forward_pressed = true;
        controller.update_camera(1.0);

        // Camera should move toward target (negative Z direction)
        assert!(controller.get_camera().eye.z < initial_eye.z);
    }

    #[test]
    fn test_update_camera_backward_movement() {
        let camera = create_test_camera();
        let initial_eye = camera.eye;
        let mut controller = CameraController::new(5.0, 0.5, camera);

        controller.is_backward_pressed = true;
        controller.update_camera(1.0);

        // Camera should move away from target (positive Z direction)
        assert!(controller.get_camera().eye.z > initial_eye.z);
    }

    #[test]
    fn test_update_camera_left_strafe() {
        let camera = create_test_camera();
        let initial_eye = camera.eye;
        let mut controller = CameraController::new(5.0, 0.5, camera);

        controller.is_left_pressed = true;
        controller.update_camera(1.0);

        // Camera should move left (negative X direction when looking at origin)
        assert!(controller.get_camera().eye.x > initial_eye.x);
    }

    #[test]
    fn test_update_camera_right_strafe() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 0.5, camera);
        let initial_eye = controller.get_camera().eye;

        controller.is_right_pressed = true;
        controller.update_camera(10.0);
        // Camera should move right (positive X direction when looking at origin)
        assert!(controller.get_camera().eye.x < initial_eye.x,);
    }

    #[test]
    fn test_update_camera_respects_delta_time() {
        let camera1 = create_test_camera();
        let mut controller = CameraController::new(5.0, 0.5, camera1);
        let camera2 = create_test_camera();
        let mut controller2 = CameraController::new(5.0, 0.5, camera2);

        controller.is_forward_pressed = true;
        controller2.is_forward_pressed = true;
        controller.update_camera(0.1);
        controller2.update_camera(0.2);

        // camera2 should have moved twice as far as camera1
        let distance1 = (controller.get_camera().eye - Vec3::new(0.0, 0.0, 10.0)).length();
        let distance2 = (controller2.get_camera().eye - Vec3::new(0.0, 0.0, 10.0)).length();
        assert!(
            (distance2 - distance1 * 2.0).abs() < 0.001,
            "{:?}, {:?}",
            distance1,
            distance2
        );
    }

    #[test]
    fn test_update_camera_no_movement_when_no_keys_pressed() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 0.5, camera);
        let initial_eye = controller.get_camera().eye.clone();
        controller.update_camera(1.0);

        assert_eq!(controller.get_camera().eye, initial_eye);
    }

    #[test]
    fn test_update_camera_forward_stops_when_too_close_to_target() {
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 0.1), // very close to target
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            16.0 / 9.0,
            45.0_f32.to_radians(),
            0.1,
            100.0,
        );

        let mut controller = CameraController::new(100.0, 0.5, camera);
        let initial_eye = controller.get_camera().eye;

        controller.is_forward_pressed = true;
        controller.update_camera(1.0);

        // Camera should not move because forward_mag <= speed
        assert_eq!(controller.get_camera().eye, initial_eye);
    }

    #[test]
    fn test_update_camera_maintains_distance_from_target_during_strafe() {
        let camera = create_test_camera();
        let initial_distance = (camera.eye - camera.target).length();
        let mut controller = CameraController::new(5.0, 0.5, camera);

        controller.is_left_pressed = true;
        controller.update_camera(0.1);
        let camera1 = controller.get_camera();
        let final_distance = (camera1.eye - camera1.target).length();

        // Distance should be approximately the same (within floating point tolerance)
        assert!((initial_distance - final_distance).abs() < 0.001);
    }

    // Helper function to create MouseDelta for testing
    fn create_mouse_delta(delta_x: f64, delta_y: f64, is_clicked: bool) -> MouseDelta {
        MouseDelta {
            delta_position: MovementDelta::new(delta_x, delta_y),
            state: MouseState::new(
                MouseButton::LEFT,
                if is_clicked {
                    MouseAction::CLICKED
                } else {
                    MouseAction::RELEASED
                },
            ),
            is_mouse_on_window: true,
            position: MousePosition::new(0.0, 0.0),
        }
    }

    #[test]
    fn test_delta_to_rotation_conversion_yaw() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 1.0, camera);
        let initial_yaw = *controller.yaw;

        let mouse_delta = create_mouse_delta(10.0, 0.0, true);
        controller.rotate(&mouse_delta).unwrap();

        // Yaw should have changed due to positive X delta
        // Note: Due to smoothing, the change won't be exactly 10.0
        // First movement with smoothing_factor 0.5: smoothed = 0.0 * 0.5 + 10.0 * 0.5 = 5.0
        assert!(
            *controller.yaw > initial_yaw,
            "Yaw should increase with positive X delta"
        );
    }

    #[test]
    fn test_delta_to_rotation_conversion_pitch() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 1.0, camera);
        let initial_pitch = *controller.pitch;

        let mouse_delta = create_mouse_delta(0.0, 10.0, true);
        controller.rotate(&mouse_delta).unwrap();

        // Pitch should decrease due to positive Y delta (inverted Y-axis)
        // First movement with smoothing_factor 0.5: smoothed = 0.0 * 0.5 + 10.0 * 0.5 = 5.0
        assert!(
            *controller.pitch < initial_pitch,
            "Pitch should decrease with positive Y delta (inverted Y-axis)"
        );
    }

    #[test]
    fn test_delta_to_rotation_negative_values() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 1.0, camera);
        let initial_yaw = *controller.yaw;
        let initial_pitch = *controller.pitch;

        let mouse_delta = create_mouse_delta(-10.0, -10.0, true);
        controller.rotate(&mouse_delta).unwrap();

        // Negative X delta should decrease yaw
        assert!(
            *controller.yaw < initial_yaw,
            "Yaw should decrease with negative X delta"
        );
        // Negative Y delta should increase pitch (inverted Y-axis: subtract negative = add)
        assert!(
            *controller.pitch > initial_pitch,
            "Pitch should increase with negative Y delta (inverted Y-axis)"
        );
    }

    #[test]
    fn test_rotation_only_when_mouse_clicked() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 1.0, camera);
        let initial_yaw = *controller.yaw;
        let initial_pitch = *controller.pitch;

        // Mouse delta with button released should not rotate
        let mouse_delta = create_mouse_delta(10.0, 10.0, false);
        controller.rotate(&mouse_delta).unwrap();

        assert_eq!(
            *controller.yaw, initial_yaw,
            "Yaw should not change when mouse button is released"
        );
        assert_eq!(
            *controller.pitch, initial_pitch,
            "Pitch should not change when mouse button is released"
        );
    }

    #[test]
    fn test_sensitivity_scaling_double_sensitivity() {
        let camera1 = create_test_camera();
        let camera2 = create_test_camera();
        let mut controller_low = CameraController::new(5.0, 0.5, camera1);
        let mut controller_high = CameraController::new(5.0, 1.0, camera2);

        let mouse_delta = create_mouse_delta(10.0, 10.0, true);

        // Apply same delta to both controllers
        controller_low.rotate(&mouse_delta).unwrap();
        controller_high.rotate(&mouse_delta).unwrap();

        // Controller with 2x sensitivity should rotate more (accounting for smoothing)
        let yaw_change_low = *controller_low.yaw - (-PI / 2.0);
        let yaw_change_high = *controller_high.yaw - (-PI / 2.0);

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
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 0.0, camera);
        let initial_yaw = *controller.yaw;
        let initial_pitch = *controller.pitch;

        let mouse_delta = create_mouse_delta(100.0, 100.0, true);
        controller.rotate(&mouse_delta).unwrap();

        // With zero sensitivity, no rotation should occur even with large delta
        assert_eq!(
            *controller.yaw, initial_yaw,
            "Yaw should not change with zero sensitivity"
        );
        assert_eq!(
            *controller.pitch, initial_pitch,
            "Pitch should not change with zero sensitivity"
        );
    }

    #[test]
    fn test_pitch_clamping_at_upper_limit() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 1.0, camera);

        // Apply large upward rotation to exceed pitch limit
        for _ in 0..100 {
            let mouse_delta = create_mouse_delta(0.0, 10.0, true);
            controller.rotate(&mouse_delta).unwrap();
        }

        // Pitch should be clamped to max (89 degrees)
        let max_pitch = 89.0_f32.to_radians();
        assert!(
            *controller.pitch <= max_pitch,
            "Pitch should be clamped at upper limit. Got: {}, Max: {}",
            *controller.pitch,
            max_pitch
        );
    }

    #[test]
    fn test_pitch_clamping_at_lower_limit() {
        let camera = create_test_camera();
        let mut controller = CameraController::new(5.0, 1.0, camera);

        // Apply large downward rotation to exceed pitch limit
        for _ in 0..100 {
            let mouse_delta = create_mouse_delta(0.0, -10.0, true);
            controller.rotate(&mouse_delta).unwrap();
        }

        // Pitch should be clamped to min (-89 degrees)
        let min_pitch = -89.0_f32.to_radians();
        assert!(
            *controller.pitch >= min_pitch,
            "Pitch should be clamped at lower limit. Got: {}, Min: {}",
            *controller.pitch,
            min_pitch
        );
    }
}
