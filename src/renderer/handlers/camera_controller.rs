use glam::Vec3;
use log::debug;

use crate::renderer::{
    actions::{Action, CameraActions},
    animator::trajectory::calculate_direction_vector,
    components::camera::Camera,
    types::{DeltaTime, mouse_delta::MouseDelta},
};

#[derive(Debug)]
pub enum CameraMode {
    FLY,
    PAN,
    ORBIT,
}

#[derive(Debug)]
struct CameraAxes {
    forward: Vec3,
    forward_mag: f32,
    right: Vec3,
    view_up: Vec3,
}

#[derive(Debug)]
pub struct CameraController {
    pub camera_mode: CameraMode,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_speed_modifier_pressed: bool,
    is_slow_modifier_pressed: bool,
    is_mouse_dragging: bool,
}

impl CameraController {
    pub fn new(camera_mode: CameraMode) -> CameraController {
        Self {
            camera_mode,
            is_backward_pressed: false,
            is_forward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            is_speed_modifier_pressed: false,
            is_slow_modifier_pressed: false,
            is_mouse_dragging: false,
        }
    }

    pub fn mouse_movement(
        &mut self,
        camera: &mut Camera,
        mouse_delta: &MouseDelta,
        delta_time: DeltaTime,
    ) {
        if self.is_mouse_dragging {
            match self.camera_mode {
                CameraMode::PAN => {
                    let delta_x = mouse_delta.delta_position.x() as f32;
                    let delta_y = mouse_delta.delta_position.y() as f32;
                    let axes = self.get_axes(camera);

                    let offset =
                        Self::calculate_pan_offset(delta_x, delta_y, &axes, camera.sensitivity);
                    camera.eye += offset;
                    camera.target += offset;
                }
                _ => {
                    let yaw_delta = mouse_delta.delta_position.x() as f32;
                    let pitch_delta = mouse_delta.delta_position.y() as f32;
                    camera.move_camera(yaw_delta, pitch_delta);
                }
            }
        }
    }

    pub fn handle_action(&mut self, action: &Action, is_pressed: bool) {
        debug!("{:?}", action);
        match action {
            Action::Camera(camera_action) => match camera_action {
                CameraActions::Forwards => self.is_forward_pressed = is_pressed,
                CameraActions::Backwards => self.is_backward_pressed = is_pressed,
                CameraActions::Left => self.is_left_pressed = is_pressed,
                CameraActions::Right => self.is_right_pressed = is_pressed,
                CameraActions::Up => self.is_up_pressed = is_pressed,
                CameraActions::Down => self.is_down_pressed = is_pressed,
                CameraActions::SpeedModifier => self.is_speed_modifier_pressed = is_pressed,
                CameraActions::SlowModifier => self.is_slow_modifier_pressed = is_pressed,
                CameraActions::Drag => self.is_mouse_dragging = is_pressed,
            },
        }
    }

    pub fn update_camera_with_keyboard(&self, camera: &mut Camera, delta_time: DeltaTime) {
        let axes = self.get_axes(camera);
        let speed = self.adjust_speed(camera.speed * delta_time);
        let movement = self.movement_calculcation(camera, axes, speed);
        self.update_camera_with_movement(camera, &movement);
    }

    fn update_camera_with_movement(&self, camera: &mut Camera, movement: &Vec3) {
        match self.camera_mode {
            CameraMode::ORBIT => camera.eye += movement,
            _ => {
                camera.eye += movement;
                camera.target += movement;
            }
        }
    }

    fn get_axes(&self, camera: &Camera) -> CameraAxes {
        let (forward, forward_mag) = match self.camera_mode {
            CameraMode::ORBIT => {
                let forward = camera.target - camera.eye;
                (forward.normalize(), forward.length())
            }
            CameraMode::FLY => (calculate_direction_vector(*camera.yaw, *camera.pitch), 1.0),
            CameraMode::PAN => {
                let forward = camera.target - camera.eye;
                (forward.normalize(), forward.length())
            }
        };

        let right = forward.cross(camera.up).normalize();
        let view_up = right.cross(forward).normalize();
        CameraAxes {
            forward,
            forward_mag,
            right,
            view_up,
        }
    }

    fn calculate_pan_offset(
        delta_x: f32,
        delta_y: f32,
        axes: &CameraAxes,
        sensitivity: f32,
    ) -> Vec3 {
        axes.view_up * (delta_y * sensitivity) - axes.right * (delta_x * sensitivity)
    }

    fn movement_calculcation(&self, camera: &Camera, axes: CameraAxes, speed: f32) -> Vec3 {
        let mut movement = Vec3::ZERO;
        self.calculate_forwards_movement(
            &axes.forward,
            axes.forward_mag,
            &axes.view_up,
            speed,
            &mut movement,
        );
        self.calculate_backwards_movement(&axes.forward, &axes.view_up, speed, &mut movement);
        self.calculate_right_movement(&axes.right, speed, &mut movement);
        self.calculate_left_movement(&axes.right, speed, &mut movement);
        self.calculate_up_movement(&camera.up, speed, &mut movement);
        self.calculate_down_movement(&camera.up, speed, &mut movement);
        movement
    }

    fn calculate_up_movement(&self, up: &Vec3, speed: f32, movement: &mut Vec3) {
        if self.is_up_pressed {
            *movement += up * speed;
        }
    }

    fn calculate_down_movement(&self, up: &Vec3, speed: f32, movement: &mut Vec3) {
        if self.is_down_pressed {
            *movement -= up * speed;
        }
    }

    fn calculate_forwards_movement(
        &self,
        forward: &Vec3,
        forward_mag: f32,
        view_up: &Vec3,
        speed: f32,
        movement: &mut Vec3,
    ) {
        if self.is_forward_pressed {
            match self.camera_mode {
                CameraMode::FLY => *movement += forward * speed,
                CameraMode::ORBIT => {
                    if forward_mag > speed {
                        *movement += forward * speed;
                    }
                }
                CameraMode::PAN => *movement += view_up * speed,
            }
        }
    }

    fn calculate_left_movement(&self, right: &Vec3, speed_multiplier: f32, movement: &mut Vec3) {
        if self.is_left_pressed {
            match self.camera_mode {
                CameraMode::PAN => *movement -= right * speed_multiplier,
                CameraMode::FLY => *movement -= right * speed_multiplier,
                CameraMode::ORBIT => *movement -= right * speed_multiplier,
            }
        }
    }
    fn calculate_right_movement(&self, right: &Vec3, speed_multiplier: f32, movement: &mut Vec3) {
        if self.is_right_pressed {
            match self.camera_mode {
                CameraMode::PAN => *movement += right * speed_multiplier,
                CameraMode::FLY => *movement += right * speed_multiplier,
                CameraMode::ORBIT => *movement += right * speed_multiplier,
            }
        }
    }

    fn calculate_backwards_movement(
        &self,
        forward: &Vec3,
        view_up: &Vec3,
        speed_multiplier: f32,
        movement: &mut Vec3,
    ) {
        if self.is_backward_pressed {
            match self.camera_mode {
                CameraMode::PAN => *movement -= view_up * speed_multiplier,
                CameraMode::FLY => *movement -= forward * speed_multiplier,
                CameraMode::ORBIT => *movement -= forward * speed_multiplier,
            }
        }
    }

    fn adjust_speed(&self, mut speed: f32) -> f32 {
        if self.is_slow_modifier_pressed {
            speed *= 0.5;
        }
        if self.is_speed_modifier_pressed {
            speed *= 2.0;
        }
        speed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::types::{
        camera::{Pitch, Yaw},
        mouse_delta::{MouseAction, MouseButton, MousePosition, MouseState, MovementDelta},
    };
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
        let controller = CameraController::new(CameraMode::FLY);
        assert!(!controller.is_backward_pressed);
        assert!(!controller.is_forward_pressed);
        assert!(!controller.is_left_pressed);
        assert!(!controller.is_right_pressed);
        assert!(!controller.is_up_pressed);
        assert!(!controller.is_down_pressed);
    }

    #[test]
    fn test_handle_key_w_sets_forward_pressed() {
        let mut controller = CameraController::new(CameraMode::FLY);
        controller.handle_action(&Action::Camera(CameraActions::Forwards), true);
        assert!(controller.is_forward_pressed);
    }

    #[test]
    fn test_handle_key_s_sets_backward_pressed() {
        let mut controller = CameraController::new(CameraMode::FLY);
        controller.handle_action(&Action::Camera(CameraActions::Backwards), true);
        assert!(controller.is_backward_pressed);
    }

    #[test]
    fn test_handle_key_a_sets_left_pressed() {
        let mut controller = CameraController::new(CameraMode::FLY);
        controller.handle_action(&Action::Camera(CameraActions::Left), true);
        assert!(controller.is_left_pressed);
    }

    #[test]
    fn test_handle_key_d_sets_right_pressed() {
        let mut controller = CameraController::new(CameraMode::FLY);
        controller.handle_action(&Action::Camera(CameraActions::Right), true);
        assert!(controller.is_right_pressed);
    }

    #[test]
    fn test_handle_key_space_sets_up_pressed() {
        let mut controller = CameraController::new(CameraMode::FLY);
        controller.handle_action(&Action::Camera(CameraActions::Up), true);
        assert!(controller.is_up_pressed);
    }

    #[test]
    fn test_handle_key_ctrl_sets_down_pressed() {
        let mut controller = CameraController::new(CameraMode::FLY);
        controller.handle_action(&Action::Camera(CameraActions::Down), true);
        assert!(controller.is_down_pressed);
    }

    #[test]
    fn test_handle_key_release_clears_state() {
        let mut controller = CameraController::new(CameraMode::ORBIT);

        controller.handle_action(&Action::Camera(CameraActions::Forwards), true);
        assert!(controller.is_forward_pressed);

        controller.handle_action(&Action::Camera(CameraActions::Forwards), false);
        assert!(!controller.is_forward_pressed);
    }

    #[test]
    fn test_update_camera_forward_movement() {
        let mut camera = create_test_camera();
        let initial_eye = camera.eye;

        let mut controller = CameraController::new(CameraMode::ORBIT);
        controller.is_forward_pressed = true;
        controller.update_camera_with_keyboard(&mut camera, 0.1); // Smaller delta to avoid overshooting

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
        let mut controller = CameraController::new(CameraMode::ORBIT);

        controller.is_backward_pressed = true;
        controller.update_camera_with_keyboard(&mut camera, 1.0);

        // Camera should move away from target (positive Z direction)
        assert!(camera.eye.z > initial_eye.z);
    }

    #[test]
    fn test_update_camera_left_strafe() {
        let mut camera = create_test_camera();
        let initial_eye = camera.eye;
        let mut controller = CameraController::new(CameraMode::ORBIT);

        controller.is_left_pressed = true;
        controller.update_camera_with_keyboard(&mut camera, 1.0);

        assert!(camera.eye.x < initial_eye.x);
    }

    #[test]
    fn test_update_camera_right_strafe() {
        let mut camera = create_test_camera();
        let mut controller = CameraController::new(CameraMode::ORBIT);
        let initial_eye = camera.eye;

        controller.is_right_pressed = true;
        controller.update_camera_with_keyboard(&mut camera, 10.0);
        assert!(camera.eye.x > initial_eye.x,);
    }

    #[test]
    fn test_update_camera_up_movement() {
        let mut camera = create_test_camera();
        let mut controller = CameraController::new(CameraMode::ORBIT);
        let initial_eye = camera.eye;

        controller.is_up_pressed = true;
        controller.update_camera_with_keyboard(&mut camera, 1.0);
        assert!(camera.eye.y > initial_eye.y);
    }

    #[test]
    fn test_update_camera_down_movement() {
        let mut camera = create_test_camera();
        let mut controller = CameraController::new(CameraMode::ORBIT);
        let initial_eye = camera.eye;

        controller.is_down_pressed = true;
        controller.update_camera_with_keyboard(&mut camera, 1.0);
        assert!(camera.eye.y < initial_eye.y);
    }

    #[test]
    fn test_update_camera_respects_delta_time() {
        let mut camera1 = create_test_camera();
        let mut camera2 = create_test_camera();
        let mut controller = CameraController::new(CameraMode::ORBIT);
        controller.is_forward_pressed = true;
        controller.update_camera_with_keyboard(&mut camera1, 0.1);
        controller.update_camera_with_keyboard(&mut camera2, 0.2);

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
        let mut controller = CameraController::new(CameraMode::ORBIT);
        let initial_eye = camera.eye.clone();
        controller.update_camera_with_keyboard(&mut camera, 1.0);
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

        let mut controller = CameraController::new(CameraMode::ORBIT);
        let initial_eye = camera.eye;

        controller.is_forward_pressed = true;
        controller.update_camera_with_keyboard(&mut camera, 1.0);

        // Camera should not move because forward_mag <= speed
        assert_eq!(camera.eye, initial_eye);
    }

    #[test]
    fn test_update_camera_strafe_changes_position() {
        let mut camera = create_test_camera();
        let initial_eye = camera.eye;
        let mut controller = CameraController::new(CameraMode::ORBIT);

        controller.is_left_pressed = true;
        controller.update_camera_with_keyboard(&mut camera, 0.1);

        assert!(camera.eye.x < initial_eye.x);
    }

    #[test]
    fn test_speed_modifier_doubles_speed() {
        let mut camera1 = create_test_camera();
        let mut camera2 = create_test_camera();
        let mut controller = CameraController::new(CameraMode::FLY);

        controller.is_forward_pressed = true;
        controller.update_camera_with_keyboard(&mut camera1, 0.1);

        controller.is_speed_modifier_pressed = true;
        controller.update_camera_with_keyboard(&mut camera2, 0.1);

        let distance1 = (camera1.eye - Vec3::new(0.0, 0.0, 10.0)).length();
        let distance2 = (camera2.eye - Vec3::new(0.0, 0.0, 10.0)).length();

        assert!(
            (distance2 - distance1 * 2.0).abs() < 0.001,
            "Speed modifier should double the distance. Normal: {}, Boosted: {}",
            distance1,
            distance2
        );
    }

    #[test]
    fn test_slow_modifier_halves_speed() {
        let mut camera1 = create_test_camera();
        let mut camera2 = create_test_camera();
        let mut controller = CameraController::new(CameraMode::FLY);

        controller.is_forward_pressed = true;
        controller.update_camera_with_keyboard(&mut camera1, 0.1);

        controller.is_slow_modifier_pressed = true;
        controller.update_camera_with_keyboard(&mut camera2, 0.1);

        let distance1 = (camera1.eye - Vec3::new(0.0, 0.0, 10.0)).length();
        let distance2 = (camera2.eye - Vec3::new(0.0, 0.0, 10.0)).length();

        assert!(
            (distance2 - distance1 * 0.5).abs() < 0.001,
            "Slow modifier should halve the distance. Normal: {}, Slowed: {}",
            distance1,
            distance2
        );
    }

    #[test]
    fn test_calculate_pan_offset_uses_mouse_delta_and_sensitivity() {
        let axes = CameraAxes {
            forward: Vec3::new(0.0, 0.0, -1.0),
            forward_mag: 1.0,
            right: Vec3::X,
            view_up: Vec3::Y,
        };

        let offset = CameraController::calculate_pan_offset(2.0, -4.0, &axes, 0.25);

        assert_eq!(offset, Vec3::new(-0.5, -1.0, 0.0));
    }

    #[test]
    fn test_calculate_pan_offset_horizontal_drag_moves_opposite_right_axis() {
        let axes = CameraAxes {
            forward: Vec3::new(0.0, 0.0, -1.0),
            forward_mag: 1.0,
            right: Vec3::X,
            view_up: Vec3::Y,
        };

        let offset = CameraController::calculate_pan_offset(1.0, 0.0, &axes, 1.0);

        assert_eq!(offset, -Vec3::X);
    }

    #[test]
    fn test_rotation_only_when_mouse_clicked() {
        let mut camera = create_test_camera();
        let mut controller = CameraController::new(CameraMode::FLY);

        let initial_yaw = camera.yaw;
        let initial_pitch = camera.pitch;

        // Mouse delta with button released should not rotate
        controller.mouse_movement(
            &mut camera,
            &MouseDelta {
                delta_position: MovementDelta::new(10.0, 10.0),
                state: MouseState::new(MouseButton::Left, MouseAction::Clicked),
                is_mouse_on_window: false,
                position: MousePosition::new(0.0, 0.0),
            },
            0.1,
        );

        assert_eq!(
            *camera.yaw, *initial_yaw,
            "Yaw should not change when mouse button is released"
        );
        assert_eq!(
            *camera.pitch, *initial_pitch,
            "Pitch should not change when mouse button is released"
        );
    }
}
