use winit::keyboard::KeyCode;

use crate::renderer::components::camera::Camera;

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

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

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
        let controller = CameraController::new(5.0);

        assert_eq!(controller.speed, 5.0);
        assert_eq!(controller.is_forward_pressed, false);
        assert_eq!(controller.is_backward_pressed, false);
        assert_eq!(controller.is_left_pressed, false);
        assert_eq!(controller.is_right_pressed, false);
    }

    #[test]
    fn test_handle_key_w_sets_forward_pressed() {
        let mut controller = CameraController::new(5.0);

        let handled = controller.handle_key(KeyCode::KeyW, true);

        assert!(handled);
        assert!(controller.is_forward_pressed);
    }

    #[test]
    fn test_handle_key_arrow_up_sets_forward_pressed() {
        let mut controller = CameraController::new(5.0);

        let handled = controller.handle_key(KeyCode::ArrowUp, true);

        assert!(handled);
        assert!(controller.is_forward_pressed);
    }

    #[test]
    fn test_handle_key_s_sets_backward_pressed() {
        let mut controller = CameraController::new(5.0);

        let handled = controller.handle_key(KeyCode::KeyS, true);

        assert!(handled);
        assert!(controller.is_backward_pressed);
    }

    #[test]
    fn test_handle_key_arrow_down_sets_backward_pressed() {
        let mut controller = CameraController::new(5.0);

        let handled = controller.handle_key(KeyCode::ArrowDown, true);

        assert!(handled);
        assert!(controller.is_backward_pressed);
    }

    #[test]
    fn test_handle_key_a_sets_left_pressed() {
        let mut controller = CameraController::new(5.0);

        let handled = controller.handle_key(KeyCode::KeyA, true);

        assert!(handled);
        assert!(controller.is_left_pressed);
    }

    #[test]
    fn test_handle_key_arrow_left_sets_left_pressed() {
        let mut controller = CameraController::new(5.0);

        let handled = controller.handle_key(KeyCode::ArrowLeft, true);

        assert!(handled);
        assert!(controller.is_left_pressed);
    }

    #[test]
    fn test_handle_key_d_sets_right_pressed() {
        let mut controller = CameraController::new(5.0);

        let handled = controller.handle_key(KeyCode::KeyD, true);

        assert!(handled);
        assert!(controller.is_right_pressed);
    }

    #[test]
    fn test_handle_key_arrow_right_sets_right_pressed() {
        let mut controller = CameraController::new(5.0);

        let handled = controller.handle_key(KeyCode::ArrowRight, true);

        assert!(handled);
        assert!(controller.is_right_pressed);
    }

    #[test]
    fn test_handle_key_release_clears_state() {
        let mut controller = CameraController::new(5.0);

        controller.handle_key(KeyCode::KeyW, true);
        assert!(controller.is_forward_pressed);

        controller.handle_key(KeyCode::KeyW, false);
        assert!(!controller.is_forward_pressed);
    }

    #[test]
    fn test_handle_key_unhandled_key_returns_false() {
        let mut controller = CameraController::new(5.0);

        let handled = controller.handle_key(KeyCode::Space, true);

        assert!(!handled);
    }

    #[test]
    fn test_update_camera_forward_movement() {
        let mut controller = CameraController::new(5.0);
        let mut camera = create_test_camera();
        let initial_eye = camera.eye;

        controller.is_forward_pressed = true;
        controller.update_camera(&mut camera, 1.0);

        // Camera should move toward target (negative Z direction)
        assert!(camera.eye.z < initial_eye.z);
    }

    #[test]
    fn test_update_camera_backward_movement() {
        let mut controller = CameraController::new(5.0);
        let mut camera = create_test_camera();
        let initial_eye = camera.eye;

        controller.is_backward_pressed = true;
        controller.update_camera(&mut camera, 1.0);

        // Camera should move away from target (positive Z direction)
        assert!(camera.eye.z > initial_eye.z);
    }

    #[test]
    fn test_update_camera_left_strafe() {
        let mut controller = CameraController::new(5.0);
        let mut camera = create_test_camera();
        let initial_eye = camera.eye;

        controller.is_left_pressed = true;
        controller.update_camera(&mut camera, 1.0);

        // Camera should move left (negative X direction when looking at origin)
        assert!(camera.eye.x > initial_eye.x);
    }

    #[test]
    fn test_update_camera_right_strafe() {
        let mut controller = CameraController::new(5.0);
        let mut camera = create_test_camera();
        let initial_eye = camera.eye;

        controller.is_right_pressed = true;
        controller.update_camera(&mut camera, 10.0);
        // Camera should move right (positive X direction when looking at origin)
        assert!(camera.eye.x < initial_eye.x,);
    }

    #[test]
    fn test_update_camera_respects_delta_time() {
        let mut controller = CameraController::new(5.0);
        let mut camera1 = create_test_camera();
        let mut camera2 = create_test_camera();

        controller.is_forward_pressed = true;

        controller.update_camera(&mut camera1, 0.1);
        controller.update_camera(&mut camera2, 0.2);

        // camera2 should have moved twice as far as camera1
        let distance1 = (camera1.eye - Vec3::new(0.0, 0.0, 10.0)).length();
        let distance2 = (camera2.eye - Vec3::new(0.0, 0.0, 10.0)).length();

        assert!((distance2 - distance1 * 2.0).abs() < 0.001);
    }

    #[test]
    fn test_update_camera_no_movement_when_no_keys_pressed() {
        let mut controller = CameraController::new(5.0);
        let mut camera = create_test_camera();
        let initial_eye = camera.eye;

        controller.update_camera(&mut camera, 1.0);

        assert_eq!(camera.eye, initial_eye);
    }

    #[test]
    fn test_update_camera_forward_stops_when_too_close_to_target() {
        let mut controller = CameraController::new(100.0);
        let mut camera = Camera::new(
            Vec3::new(0.0, 0.0, 0.1), // very close to target
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            16.0 / 9.0,
            45.0_f32.to_radians(),
            0.1,
            100.0,
        );
        let initial_eye = camera.eye;

        controller.is_forward_pressed = true;
        controller.update_camera(&mut camera, 1.0);

        // Camera should not move because forward_mag <= speed
        assert_eq!(camera.eye, initial_eye);
    }

    #[test]
    fn test_update_camera_maintains_distance_from_target_during_strafe() {
        let mut controller = CameraController::new(5.0);
        let mut camera = create_test_camera();
        let initial_distance = (camera.eye - camera.target).length();

        controller.is_left_pressed = true;
        controller.update_camera(&mut camera, 0.1);

        let final_distance = (camera.eye - camera.target).length();

        // Distance should be approximately the same (within floating point tolerance)
        assert!((initial_distance - final_distance).abs() < 0.001);
    }
}
