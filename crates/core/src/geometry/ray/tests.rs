use glam::{Vec2, Vec3};

use crate::{
    components::camera::camera::Camera,
    geometry::ray::{ndc_to_world, ray_from_screen, screen_to_ndc},
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
        1.0,
        0.5,
    )
}

fn test_size() -> Size {
    Size {
        width: 800,
        height: 600,
    }
}

fn direction_from_click(camera: &Camera, x: f32, y: f32) -> Vec3 {
    ray_from_screen(camera, x, y, test_size())
        .unwrap()
        .direction()
        .normalize()
}

#[test]
fn screen_to_ndc_maps_center_to_origin() {
    let ndc = screen_to_ndc(400.0, 300.0, test_size()).unwrap();

    assert_eq!(ndc.x, 0.0);
    assert_eq!(ndc.y, 0.0);
}

#[test]
fn screen_to_ndc_maps_top_left_to_negative_x_positive_y() {
    let ndc = screen_to_ndc(0.0, 0.0, test_size()).unwrap();

    assert_eq!(ndc.x, -1.0);
    assert_eq!(ndc.y, 1.0);
}

#[test]
fn screen_to_ndc_maps_bottom_right_to_positive_x_negative_y() {
    let ndc = screen_to_ndc(800.0, 600.0, test_size()).unwrap();

    assert_eq!(ndc.x, 1.0);
    assert_eq!(ndc.y, -1.0);
}

#[test]
fn screen_to_ndc_rejects_zero_size() {
    let ndc = screen_to_ndc(
        0.0,
        0.0,
        Size {
            width: 0,
            height: 0,
        },
    );

    assert!(ndc.is_none());
}

#[test]
fn ndc_to_world_unprojects_center_to_near_plane() {
    let camera = create_test_camera();
    let world = ndc_to_world(&camera, Vec2::ZERO, 0.0).unwrap();

    assert!((world.x - 0.0).abs() < 0.0001);
    assert!((world.y - 0.0).abs() < 0.0001);
    assert!((world.z - 9.9).abs() < 0.0001);
}

#[test]
fn ndc_to_world_rejects_depth_outside_clip_range() {
    let camera = create_test_camera();

    assert!(ndc_to_world(&camera, Vec2::ZERO, -0.1).is_none());
    assert!(ndc_to_world(&camera, Vec2::ZERO, 1.1).is_none());
}

#[test]
fn ray_center_point_click_test() {
    let camera = create_test_camera();
    let ray = ray_from_screen(&camera, 400.0, 300.0, test_size());

    assert!(ray.is_ok());
    let ray = ray.unwrap();
    assert_eq!(ray.origin(), camera.eye);
    let direction = ray.direction().normalize();
    assert!((direction.x - 0.0).abs() < 0.0001);
    assert!((direction.y - 0.0).abs() < 0.0001);
    assert!((direction.z + 1.0).abs() < 0.0001);
}

#[test]
fn ray_right_point_click_points_right() {
    let camera = create_test_camera();
    let direction = direction_from_click(&camera, 800.0, 300.0);

    assert!(direction.x > 0.0);
    assert!((direction.y - 0.0).abs() < 0.0001);
    assert!(direction.z < 0.0);
}

#[test]
fn ray_left_point_click_points_left() {
    let camera = create_test_camera();
    let direction = direction_from_click(&camera, 0.0, 300.0);

    assert!(direction.x < 0.0);
    assert!((direction.y - 0.0).abs() < 0.0001);
    assert!(direction.z < 0.0);
}

#[test]
fn ray_top_point_click_points_up() {
    let camera = create_test_camera();
    let direction = direction_from_click(&camera, 400.0, 0.0);

    assert!((direction.x - 0.0).abs() < 0.0001);
    assert!(direction.y > 0.0);
    assert!(direction.z < 0.0);
}

#[test]
fn ray_bottom_point_click_points_down() {
    let camera = create_test_camera();
    let direction = direction_from_click(&camera, 400.0, 600.0);

    assert!((direction.x - 0.0).abs() < 0.0001);
    assert!(direction.y < 0.0);
    assert!(direction.z < 0.0);
}
