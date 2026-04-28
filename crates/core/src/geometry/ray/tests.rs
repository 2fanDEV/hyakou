use glam::{Vec2, Vec3};

use crate::{
    components::camera::camera::Camera,
    geometry::ray::{ndc_to_world, ray_from_screen, screen_to_ndc},
    types::{
        Size,
        camera::{Pitch, Yaw},
    },
};

const EPSILON: f32 = 0.0001;

fn create_test_camera(size: Size) -> Camera {
    Camera::new(
        Vec3::new(0.0, 0.0, 10.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Camera::aspect_ratio_from_size(size),
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
        width: 1920,
        height: 1080,
    }
}

fn ray_direction(camera: &Camera, x: f32, y: f32, size: Size) -> Vec3 {
    ray_from_screen(camera, x, y, size).unwrap().direction()
}

fn assert_vec3_near(actual: Vec3, expected: Vec3) {
    assert!(
        (actual - expected).length() < EPSILON,
        "actual: {actual:?}, expected: {expected:?}"
    );
}

#[test]
fn screen_to_ndc_maps_screen_points() {
    let cases = [
        ("center", 960.0, 540.0, Vec2::ZERO),
        ("top left", 0.0, 0.0, Vec2::new(-1.0, 1.0)),
        ("bottom right", 1920.0, 1080.0, Vec2::new(1.0, -1.0)),
    ];

    for (name, x, y, expected) in cases {
        let ndc = screen_to_ndc(x, y, test_size()).unwrap();

        assert!(
            (ndc - expected).length() < EPSILON,
            "{name} NDC mismatch: actual={ndc:?}, expected={expected:?}"
        );
    }
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
    let camera = create_test_camera(test_size());
    let world = ndc_to_world(&camera, Vec2::ZERO, 0.0).unwrap();

    assert_vec3_near(world, Vec3::new(0.0, 0.0, 9.9));
}

#[test]
fn ndc_to_world_rejects_depth_outside_clip_range() {
    let camera = create_test_camera(test_size());

    assert!(ndc_to_world(&camera, Vec2::ZERO, -0.1).is_none());
    assert!(ndc_to_world(&camera, Vec2::ZERO, 1.1).is_none());
}

#[test]
fn ray_center_point_click_test() {
    let camera = create_test_camera(test_size());
    let ray = ray_from_screen(&camera, 960.0, 540.0, test_size());

    assert!(ray.is_ok());
    let ray = ray.unwrap();
    assert_eq!(ray.origin(), camera.eye);
    assert_vec3_near(ray.direction(), Vec3::new(0.0, 0.0, -1.0));
}

#[test]
fn ray_center_click_follows_transformed_camera_direction() {
    let size = test_size();
    let camera = Camera::new(
        Vec3::new(10.0, 0.0, 0.0),
        Vec3::ZERO,
        Vec3::Y,
        Camera::aspect_ratio_from_size(size),
        45.0_f32.to_radians(),
        0.1,
        100.0,
        Yaw::new(0.0),
        Pitch::new(0.0),
        20.0,
        1.0,
        0.5,
    );
    let ray = ray_from_screen(&camera, 960.0, 540.0, size).unwrap();

    assert_eq!(ray.origin(), camera.eye);
    assert_vec3_near(ray.direction(), Vec3::new(-1.0, 0.0, 0.0));
}

#[test]
fn ray_edge_clicks_point_in_expected_directions() {
    let size = test_size();
    let camera = create_test_camera(size);
    let cases = [
        ("right", 1920.0, 540.0, 1.0, 0.0),
        ("left", 0.0, 540.0, -1.0, 0.0),
        ("top", 960.0, 0.0, 0.0, 1.0),
        ("bottom", 960.0, 1080.0, 0.0, -1.0),
    ];

    for (name, x, y, expected_x_sign, expected_y_sign) in cases {
        let direction = ray_direction(&camera, x, y, size);

        if expected_x_sign == 0.0 {
            assert!(
                direction.x.abs() < EPSILON,
                "{name} ray drifted on x axis: {direction:?}"
            );
        } else {
            assert_eq!(
                direction.x.signum(),
                expected_x_sign,
                "{name} ray points wrong on x axis: {direction:?}"
            );
        }

        if expected_y_sign == 0.0 {
            assert!(
                direction.y.abs() < EPSILON,
                "{name} ray drifted on y axis: {direction:?}"
            );
        } else {
            assert_eq!(
                direction.y.signum(),
                expected_y_sign,
                "{name} ray points wrong on y axis: {direction:?}"
            );
        }

        assert!(
            direction.z < 0.0,
            "{name} ray should point forward: {direction:?}"
        );
    }
}

#[test]
fn ray_direction_matches_across_same_aspect_resolutions() {
    let small = Size {
        width: 960,
        height: 540,
    };
    let large = Size {
        width: 1920,
        height: 1080,
    };
    let small_camera = create_test_camera(small);
    let large_camera = create_test_camera(large);
    let cases = [
        ("center", (480.0, 270.0), (960.0, 540.0)),
        ("right", (960.0, 270.0), (1920.0, 540.0)),
        ("left", (0.0, 270.0), (0.0, 540.0)),
        ("top", (480.0, 0.0), (960.0, 0.0)),
        ("bottom", (480.0, 540.0), (960.0, 1080.0)),
    ];

    for (name, small_click, large_click) in cases {
        let small_direction = ray_direction(&small_camera, small_click.0, small_click.1, small);
        let large_direction = ray_direction(&large_camera, large_click.0, large_click.1, large);

        assert!(
            (small_direction - large_direction).length() < EPSILON,
            "{name} ray differs: small={small_direction:?}, large={large_direction:?}"
        );
    }
}
