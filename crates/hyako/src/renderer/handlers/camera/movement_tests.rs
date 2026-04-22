use glam::Vec3;
use hyakou_core::types::{
    camera::{Pitch, Yaw},
    mouse_delta::{MouseAction, MouseButton, MousePosition, MouseState, MovementDelta},
};

use super::*;

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
        Yaw::new(-PI / 2.0),
        Pitch::new(0.0),
        20.0,
        0.5,
        0.5,
    )
}

#[test]
fn test_new_controller_has_correct_initial_state() {
    let controller = CameraMovementHandler::new();
    assert!(!controller.is_backward_pressed);
    assert!(!controller.is_forward_pressed);
    assert!(!controller.is_left_pressed);
    assert!(!controller.is_right_pressed);
    assert!(!controller.is_up_pressed);
    assert!(!controller.is_down_pressed);
}

#[test]
fn test_handle_key_w_sets_forward_pressed() {
    let mut controller = CameraMovementHandler::new();
    controller.handle_action(&Action::Camera(CameraActions::Forwards), true);
    assert!(controller.is_forward_pressed);
}

#[test]
fn test_handle_key_s_sets_backward_pressed() {
    let mut controller = CameraMovementHandler::new();
    controller.handle_action(&Action::Camera(CameraActions::Backwards), true);
    assert!(controller.is_backward_pressed);
}

#[test]
fn test_handle_key_a_sets_left_pressed() {
    let mut controller = CameraMovementHandler::new();
    controller.handle_action(&Action::Camera(CameraActions::Left), true);
    assert!(controller.is_left_pressed);
}

#[test]
fn test_handle_key_d_sets_right_pressed() {
    let mut controller = CameraMovementHandler::new();
    controller.handle_action(&Action::Camera(CameraActions::Right), true);
    assert!(controller.is_right_pressed);
}

#[test]
fn test_handle_key_space_sets_up_pressed() {
    let mut controller = CameraMovementHandler::new();
    controller.handle_action(&Action::Camera(CameraActions::Up), true);
    assert!(controller.is_up_pressed);
}

#[test]
fn test_handle_key_ctrl_sets_down_pressed() {
    let mut controller = CameraMovementHandler::new();
    controller.handle_action(&Action::Camera(CameraActions::Down), true);
    assert!(controller.is_down_pressed);
}

#[test]
fn test_handle_key_release_clears_state() {
    let mut controller = CameraMovementHandler::new();

    controller.handle_action(&Action::Camera(CameraActions::Forwards), true);
    assert!(controller.is_forward_pressed);

    controller.handle_action(&Action::Camera(CameraActions::Forwards), false);
    assert!(!controller.is_forward_pressed);
}

#[test]
fn test_update_camera_forward_movement() {
    let mut camera = create_test_camera();
    let initial_eye = camera.eye;

    let mut controller = CameraMovementHandler::new();
    controller.is_forward_pressed = true;
    controller.update_camera_with_keyboard(&mut camera, &CameraMode::ORBIT, 0.1);

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
    let mut controller = CameraMovementHandler::new();

    controller.is_backward_pressed = true;
    controller.update_camera_with_keyboard(&mut camera, &CameraMode::ORBIT, 1.0);

    assert!(camera.eye.z > initial_eye.z);
}

#[test]
fn test_update_camera_left_strafe() {
    let mut camera = create_test_camera();
    let initial_eye = camera.eye;
    let mut controller = CameraMovementHandler::new();

    controller.is_left_pressed = true;
    controller.update_camera_with_keyboard(&mut camera, &CameraMode::ORBIT, 1.0);

    assert!(camera.eye.x < initial_eye.x);
}

#[test]
fn test_update_camera_right_strafe() {
    let mut camera = create_test_camera();
    let mut controller = CameraMovementHandler::new();
    let initial_eye = camera.eye;

    controller.is_right_pressed = true;
    controller.update_camera_with_keyboard(&mut camera, &CameraMode::ORBIT, 10.0);
    assert!(camera.eye.x > initial_eye.x);
}

#[test]
fn test_update_camera_up_movement() {
    let mut camera = create_test_camera();
    let mut controller = CameraMovementHandler::new();
    let initial_eye = camera.eye;

    controller.is_up_pressed = true;
    controller.update_camera_with_keyboard(&mut camera, &CameraMode::ORBIT, 1.0);
    assert!(camera.eye.y > initial_eye.y);
}

#[test]
fn test_update_camera_down_movement() {
    let mut camera = create_test_camera();
    let mut controller = CameraMovementHandler::new();
    let initial_eye = camera.eye;

    controller.is_down_pressed = true;
    controller.update_camera_with_keyboard(&mut camera, &CameraMode::ORBIT, 1.0);
    assert!(camera.eye.y < initial_eye.y);
}

#[test]
fn test_update_camera_respects_delta_time() {
    let mut camera1 = create_test_camera();
    let mut camera2 = create_test_camera();
    let mut controller = CameraMovementHandler::new();
    controller.is_forward_pressed = true;
    controller.update_camera_with_keyboard(&mut camera1, &CameraMode::ORBIT, 0.1);
    controller.update_camera_with_keyboard(&mut camera2, &CameraMode::ORBIT, 0.2);

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
    let controller = CameraMovementHandler::new();
    let initial_eye = camera.eye;
    controller.update_camera_with_keyboard(&mut camera, &CameraMode::ORBIT, 1.0);
    assert_eq!(camera.eye, initial_eye);
}

#[test]
fn test_update_camera_forward_stops_when_too_close_to_target() {
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 0.1),
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

    let mut controller = CameraMovementHandler::new();
    let initial_eye = camera.eye;

    controller.is_forward_pressed = true;
    controller.update_camera_with_keyboard(&mut camera, &CameraMode::ORBIT, 1.0);

    assert_eq!(camera.eye, initial_eye);
}

#[test]
fn test_update_camera_strafe_changes_position() {
    let mut camera = create_test_camera();
    let initial_eye = camera.eye;
    let mut controller = CameraMovementHandler::new();

    controller.is_left_pressed = true;
    controller.update_camera_with_keyboard(&mut camera, &CameraMode::ORBIT, 0.1);

    assert!(camera.eye.x < initial_eye.x);
}

#[test]
fn test_speed_modifier_doubles_speed() {
    let mut camera1 = create_test_camera();
    let mut camera2 = create_test_camera();
    let mut controller = CameraMovementHandler::new();

    controller.is_forward_pressed = true;
    controller.update_camera_with_keyboard(&mut camera1, &CameraMode::ORBIT, 0.1);

    controller.is_speed_modifier_pressed = true;
    controller.update_camera_with_keyboard(&mut camera2, &CameraMode::ORBIT, 0.1);

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
    let mut controller = CameraMovementHandler::new();

    controller.is_forward_pressed = true;
    controller.update_camera_with_keyboard(&mut camera1, &CameraMode::ORBIT, 0.1);

    controller.is_slow_modifier_pressed = true;
    controller.update_camera_with_keyboard(&mut camera2, &CameraMode::ORBIT, 0.1);

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

    let offset = CameraMovementHandler::calculate_pan_offset(2.0, -4.0, &axes, 0.25);

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

    let offset = CameraMovementHandler::calculate_pan_offset(1.0, 0.0, &axes, 1.0);

    assert_eq!(offset, -Vec3::X);
}

#[test]
fn test_rotation_only_when_mouse_clicked() {
    let mut camera = create_test_camera();
    let mut controller = CameraMovementHandler::new();

    let initial_yaw = camera.yaw;
    let initial_pitch = camera.pitch;

    controller.mouse_movement(
        &mut camera,
        &CameraMode::ORBIT,
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

#[test]
fn test_orbit_drag_keeps_target_fixed() {
    let mut camera = create_test_camera();
    let mut controller = CameraMovementHandler::new();
    controller.is_mouse_dragging = true;

    let initial_target = camera.target;
    let initial_radius = camera.eye.distance(camera.target);

    controller.mouse_movement(
        &mut camera,
        &CameraMode::ORBIT,
        &MouseDelta {
            delta_position: MovementDelta::new(10.0, -5.0),
            state: MouseState::new(MouseButton::Left, MouseAction::Clicked),
            is_mouse_on_window: true,
            position: MousePosition::new(0.0, 0.0),
        },
        0.1,
    );

    assert_eq!(camera.target, initial_target);
    assert!((camera.eye.distance(camera.target) - initial_radius).abs() < 0.001);
}

#[test]
fn test_fly_drag_updates_target_relative_to_eye() {
    let mut camera = create_test_camera();
    let mut controller = CameraMovementHandler::new();
    controller.is_mouse_dragging = true;

    controller.mouse_movement(
        &mut camera,
        &CameraMode::FLY,
        &MouseDelta {
            delta_position: MovementDelta::new(10.0, -5.0),
            state: MouseState::new(MouseButton::Left, MouseAction::Clicked),
            is_mouse_on_window: true,
            position: MousePosition::new(0.0, 0.0),
        },
        0.1,
    );

    let forward = calculate_direction_vector(*camera.yaw, *camera.pitch);
    assert!((camera.target - (camera.eye + forward)).length() < 0.001);
}
