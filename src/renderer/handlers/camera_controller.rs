use anyhow::{Ok, Result};

use crate::renderer::{
    actions::{Action, CameraActions},
    components::camera::Camera,
    types::mouse_delta::MouseDelta,
};

#[derive(Debug)]
pub struct CameraController {
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    modifier_pressed: bool,
}

impl CameraController {
    pub fn new() -> CameraController {
        Self {
            is_backward_pressed: false,
            is_forward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            modifier_pressed: false,
        }
    }

    pub fn rotate(&mut self, camera: &mut Camera, mouse_delta: &MouseDelta) -> Result<()> {
        camera.move_camera_with_mouse(mouse_delta);
        Ok(())
    }

    pub fn handle_action(&mut self, action: &Action, is_pressed: bool) {
        match action {
            Action::Camera(camera_action) => match camera_action {
                CameraActions::Forwards => {
                    self.is_forward_pressed = is_pressed;
                }
                CameraActions::Backwards => {
                    self.is_backward_pressed = is_pressed;
                }
                CameraActions::Left => {
                    self.is_left_pressed = is_pressed;
                }
                CameraActions::Right => {
                    self.is_right_pressed = is_pressed;
                }
                CameraActions::ForwardsModifier => {
                    self.is_forward_pressed = is_pressed;
                    self.modifier_pressed = is_pressed;
                }
            },
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, delta_time: f32) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.length();
        let speed = camera.camera_speed * delta_time;
        if self.is_forward_pressed && forward_mag > speed {
            let forward_speed = if self.modifier_pressed {
                speed * 2.0
            } else {
                speed
            };
            camera.eye += forward_norm * forward_speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * speed;
        }

        let right = forward_norm.cross(camera.up);
        if self.is_right_pressed {
            camera.eye += right * speed;
        }
        if self.is_left_pressed {
            camera.eye -= right * speed;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::types::camera::{Pitch, Yaw};
    use glam::Vec3;

    fn create_test_camera() -> Camera {
        use std::f32::consts::PI;
        Camera::new(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            16.0 / 9.0,
            45.0_f32.to_radians(),
            0.1,
            100.0,
            Yaw::new(-PI / 2.0), // -90° to look in -Z direction
            Pitch::new(0.0),
            20.0,
            0.5,
            0.5,
        )
    }

    #[test]
    fn test_new_controller_has_correct_initial_state() {
        let controller = CameraController::new();
        assert!(controller.is_backward_pressed == false);
        assert!(controller.is_forward_pressed == false);
        assert!(controller.is_left_pressed == false);
        assert!(controller.is_right_pressed == false);
    }

    #[test]
    fn test_handle_key_w_sets_forward_pressed() {
        let mut controller = CameraController::new();
        controller.handle_action(&Action::Camera(CameraActions::Forwards), true);
        assert!(controller.is_forward_pressed);
    }

    #[test]
    fn test_handle_key_s_sets_backward_pressed() {
        let mut controller = CameraController::new();
        controller.handle_action(&Action::Camera(CameraActions::Backwards), true);
        assert!(controller.is_backward_pressed);
    }

    #[test]
    fn test_handle_key_a_sets_left_pressed() {
        let mut controller = CameraController::new();
        controller.handle_action(&Action::Camera(CameraActions::Left), true);
        assert!(controller.is_left_pressed);
    }

    #[test]
    fn test_handle_key_d_sets_right_pressed() {
        let mut controller = CameraController::new();
        controller.handle_action(&Action::Camera(CameraActions::Right), true);
        assert!(controller.is_right_pressed);
    }

    #[test]
    fn test_handle_key_release_clears_state() {
        let mut controller = CameraController::new();

        controller.handle_action(&Action::Camera(CameraActions::Forwards), true);
        assert!(controller.is_forward_pressed);

        controller.handle_action(&Action::Camera(CameraActions::Forwards), false);
        assert!(!controller.is_forward_pressed);
    }

    #[test]
    fn test_update_camera_forward_movement() {
        let mut camera = create_test_camera();
        let initial_eye = camera.eye;

        let mut controller = CameraController::new();
        controller.is_forward_pressed = true;
        controller.update_camera(&mut camera, 0.1); // Smaller delta to avoid overshooting

        // Camera should move toward target (negative Z direction)
        assert!(
            camera.eye.z < initial_eye.z,
            "Eye should move forward (negative Z). Initial: {:?}, New: {:?}",
            initial_eye,
            camera.eye
        );
    }

    #[test]
    fn test_update_camera_backward_movement() {
        let mut camera = create_test_camera();
        let initial_eye = camera.eye;
        let mut controller = CameraController::new();

        controller.is_backward_pressed = true;
        controller.update_camera(&mut camera, 1.0);

        // Camera should move away from target (positive Z direction)
        assert!(camera.eye.z > initial_eye.z);
    }

    #[test]
    fn test_update_camera_left_strafe() {
        let mut camera = create_test_camera();
        let initial_eye = camera.eye;
        let mut controller = CameraController::new();

        controller.is_left_pressed = true;
        controller.update_camera(&mut camera, 1.0);

        assert!(camera.eye.x < initial_eye.x);
    }

    #[test]
    fn test_update_camera_right_strafe() {
        let mut camera = create_test_camera();
        let mut controller = CameraController::new();
        let initial_eye = camera.eye;

        controller.is_right_pressed = true;
        controller.update_camera(&mut camera, 10.0);
        assert!(camera.eye.x > initial_eye.x,);
    }

    #[test]
    fn test_update_camera_respects_delta_time() {
        let mut camera1 = create_test_camera();
        let mut camera2 = create_test_camera();
        let mut controller = CameraController::new();
        controller.is_forward_pressed = true;
        controller.update_camera(&mut camera1, 0.1);
        controller.update_camera(&mut camera2, 0.2);

        // camera2 should have moved twice as far as camera1
        let distance1 = (camera1.eye - Vec3::new(0.0, 0.0, 10.0)).length();
        let distance2 = (camera2.eye - Vec3::new(0.0, 0.0, 10.0)).length();
        assert!(
            (distance2 - distance1 * 2.0).abs() < 0.001,
            "{:?}, {:?}",
            distance1,
            distance2
        );
    }

    #[test]
    fn test_update_camera_no_movement_when_no_keys_pressed() {
        let mut camera = create_test_camera();
        let mut controller = CameraController::new();
        let initial_eye = camera.eye.clone();
        controller.update_camera(&mut camera, 1.0);
        assert_eq!(camera.eye, initial_eye);
    }

    #[test]
    fn test_update_camera_forward_stops_when_too_close_to_target() {
        let mut camera = Camera::new(
            Vec3::new(0.0, 0.0, 0.1), // very close to target
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            16.0 / 9.0,
            45.0_f32.to_radians(),
            0.1,
            100.0,
            Yaw::default(),
            Pitch::default(),
            20.0,
            0.5,
            0.5,
        );

        let mut controller = CameraController::new();
        let initial_eye = camera.eye;

        controller.is_forward_pressed = true;
        controller.update_camera(&mut camera, 1.0);

        // Camera should not move because forward_mag <= speed
        assert_eq!(camera.eye, initial_eye);
    }

    #[test]
    fn test_update_camera_strafe_changes_position() {
        let mut camera = create_test_camera();
        let initial_eye = camera.eye;
        let mut controller = CameraController::new();

        controller.is_left_pressed = true;
        controller.update_camera(&mut camera, 0.1);

        assert!(camera.eye.x < initial_eye.x);
    }
}
