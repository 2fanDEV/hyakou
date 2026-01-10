use anyhow::{Ok, Result};
use winit::keyboard::KeyCode;

use crate::renderer::{
    components::camera::Camera, handlers::keyboard_handler::KeyboardHandler,
    types::mouse_delta::MouseDelta,
};

#[derive(Debug)]
pub struct CameraController {
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new() -> CameraController {
        Self {
            is_backward_pressed: false,
            is_forward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn rotate(&mut self, camera: &mut Camera, mouse_delta: &MouseDelta) -> Result<()> {
        camera.move_camera_with_mouse(mouse_delta);
        Ok(())
    }

    pub fn handle_action(&mut self, key_code: KeyCode, keyboard_handler: &KeyboardHandler) -> bool {
        let mut is_pressed = keyboard_handler.is_pressed(key_code);
        match key_code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
            }
            _ => is_pressed = false,
        }
        is_pressed
    }

    pub fn update_camera(&mut self, camera: &mut Camera, delta_time: f32) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.length();
        let speed = camera.camera_speed * delta_time;
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
    use crate::renderer::handlers::keyboard_handler::{KeyState, KeyboardHandler};
    use crate::renderer::types::camera::{Pitch, Yaw};
    use glam::Vec3;

    fn create_test_keyboard_handler(key_code: KeyCode, is_pressed: bool) -> KeyboardHandler {
        let mut handler = KeyboardHandler::new();
        let key_state = if is_pressed {
            KeyState::PRESSED
        } else {
            KeyState::RELEASED
        };
        handler.handle_key_state(key_code, key_state);
        handler
    }

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
        let keyboard_handler = create_test_keyboard_handler(KeyCode::KeyW, true);

        let handled = controller.handle_action(KeyCode::KeyW, &keyboard_handler);

        assert!(handled);
        assert!(controller.is_forward_pressed);
    }

    #[test]
    fn test_handle_key_arrow_up_sets_forward_pressed() {
        let mut controller = CameraController::new();
        let keyboard_handler = create_test_keyboard_handler(KeyCode::ArrowUp, true);

        let handled = controller.handle_action(KeyCode::ArrowUp, &keyboard_handler);

        assert!(handled);
        assert!(controller.is_forward_pressed);
    }

    #[test]
    fn test_handle_key_s_sets_backward_pressed() {
        let mut controller = CameraController::new();
        let keyboard_handler = create_test_keyboard_handler(KeyCode::KeyS, true);

        let handled = controller.handle_action(KeyCode::KeyS, &keyboard_handler);

        assert!(handled);
        assert!(controller.is_backward_pressed);
    }

    #[test]
    fn test_handle_key_arrow_down_sets_backward_pressed() {
        let mut controller = CameraController::new();
        let keyboard_handler = create_test_keyboard_handler(KeyCode::ArrowDown, true);

        let handled = controller.handle_action(KeyCode::ArrowDown, &keyboard_handler);

        assert!(handled);
        assert!(controller.is_backward_pressed);
    }

    #[test]
    fn test_handle_key_a_sets_left_pressed() {
        let mut controller = CameraController::new();
        let keyboard_handler = create_test_keyboard_handler(KeyCode::KeyA, true);

        let handled = controller.handle_action(KeyCode::KeyA, &keyboard_handler);

        assert!(handled);
        assert!(controller.is_left_pressed);
    }

    #[test]
    fn test_handle_key_arrow_left_sets_left_pressed() {
        let mut controller = CameraController::new();
        let keyboard_handler = create_test_keyboard_handler(KeyCode::ArrowLeft, true);

        let handled = controller.handle_action(KeyCode::ArrowLeft, &keyboard_handler);

        assert!(handled);
        assert!(controller.is_left_pressed);
    }

    #[test]
    fn test_handle_key_d_sets_right_pressed() {
        let mut controller = CameraController::new();
        let keyboard_handler = create_test_keyboard_handler(KeyCode::KeyD, true);

        let handled = controller.handle_action(KeyCode::KeyD, &keyboard_handler);

        assert!(handled);
        assert!(controller.is_right_pressed);
    }

    #[test]
    fn test_handle_key_arrow_right_sets_right_pressed() {
        let mut controller = CameraController::new();
        let keyboard_handler = create_test_keyboard_handler(KeyCode::ArrowRight, true);

        let handled = controller.handle_action(KeyCode::ArrowRight, &keyboard_handler);

        assert!(handled);
        assert!(controller.is_right_pressed);
    }

    #[test]
    fn test_handle_key_release_clears_state() {
        let mut controller = CameraController::new();
        let keyboard_handler_pressed = create_test_keyboard_handler(KeyCode::KeyW, true);

        controller.handle_action(KeyCode::KeyW, &keyboard_handler_pressed);
        assert!(controller.is_forward_pressed);

        let keyboard_handler_released = create_test_keyboard_handler(KeyCode::KeyW, false);
        controller.handle_action(KeyCode::KeyW, &keyboard_handler_released);
        assert!(!controller.is_forward_pressed);
    }

    #[test]
    fn test_handle_key_unhandled_key_returns_false() {
        let mut controller = CameraController::new();
        let keyboard_handler = create_test_keyboard_handler(KeyCode::Space, true);

        let handled = controller.handle_action(KeyCode::Space, &keyboard_handler);

        assert!(!handled);
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

        // Camera should move left (negative X direction when looking at origin)
        assert!(camera.eye.x > initial_eye.x);
    }

    #[test]
    fn test_update_camera_right_strafe() {
        let mut camera = create_test_camera();
        let mut controller = CameraController::new();
        let initial_eye = camera.eye;

        controller.is_right_pressed = true;
        controller.update_camera(&mut camera, 10.0);
        // Camera should move right (positive X direction when looking at origin)
        assert!(camera.eye.x < initial_eye.x,);
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
    fn test_update_camera_maintains_distance_from_target_during_strafe() {
        let mut camera = create_test_camera();
        let initial_distance = (camera.eye - camera.target).length();
        let mut controller = CameraController::new();

        controller.is_left_pressed = true;
        controller.update_camera(&mut camera, 0.1);
        let final_distance = (camera.eye - camera.target).length();

        // Distance should be approximately the same (within floating point tolerance)
        assert!((initial_distance - final_distance).abs() < 0.001);
    }
}
