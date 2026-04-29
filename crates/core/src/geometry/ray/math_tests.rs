use glam::{Quat, Vec3};
use std::f32::consts::FRAC_PI_2;

use super::{TEST_EPSILON, intersect_mesh, intersect_transformed_mesh, intersect_triangle};
use crate::{
    geometry::{mesh::Mesh, ray::Ray, vertices::Vertex},
    types::transform::Transform,
};

fn vertex(position: Vec3) -> Vertex {
    Vertex {
        position,
        ..Default::default()
    }
}

fn mesh(vertices: Vec<Vertex>, indices: Vec<u32>) -> Mesh {
    Mesh::new(None, None, vertices, indices)
}

fn triangle_mesh(z: f32) -> Mesh {
    mesh(
        vec![
            vertex(Vec3::new(-1.0, -1.0, z)),
            vertex(Vec3::new(1.0, -1.0, z)),
            vertex(Vec3::new(0.0, 1.0, z)),
        ],
        vec![0, 1, 2],
    )
}

#[test]
fn ray_hits_triangle() {
    let ray = Ray::new(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
    let v0 = vertex(Vec3::new(-1.0, -1.0, 0.0));
    let v1 = vertex(Vec3::new(1.0, -1.0, 0.0));
    let v2 = vertex(Vec3::new(0.0, 1.0, 0.0));
    let hit = intersect_triangle(&ray, &v0, &v1, &v2).unwrap();

    assert!((hit - 1.0).abs() < TEST_EPSILON);
}

#[test]
fn ray_misses_triangle() {
    let ray = Ray::new(Vec3::new(2.0, 2.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
    let v0 = vertex(Vec3::new(-1.0, -1.0, 0.0));
    let v1 = vertex(Vec3::new(1.0, -1.0, 0.0));
    let v2 = vertex(Vec3::new(0.0, 1.0, 0.0));
    let hit = intersect_triangle(&ray, &v0, &v1, &v2);

    assert!(hit.is_none());
}

#[test]
fn parallel_ray_misses_triangle() {
    let ray = Ray::new(Vec3::new(0.0, 0.0, 1.0), Vec3::X);
    let v0 = vertex(Vec3::new(-1.0, -1.0, 0.0));
    let v1 = vertex(Vec3::new(1.0, -1.0, 0.0));
    let v2 = vertex(Vec3::new(0.0, 1.0, 0.0));
    let hit = intersect_triangle(&ray, &v0, &v1, &v2);

    assert!(hit.is_none());
}

#[test]
fn ray_hits_mesh_triangle() {
    let ray = Ray::new(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
    let mesh = triangle_mesh(0.0);
    let hit = intersect_mesh(&ray, &mesh).unwrap();

    assert!((hit.distance - 1.0).abs() < TEST_EPSILON);
    assert_eq!(hit.triangle_index, 0);
    assert!((hit.position - Vec3::ZERO).length() < TEST_EPSILON);
}

#[test]
fn ray_misses_mesh() {
    let ray = Ray::new(Vec3::new(2.0, 2.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
    let mesh = triangle_mesh(0.0);

    assert!(intersect_mesh(&ray, &mesh).is_none());
}

#[test]
fn ray_hits_closest_mesh_triangle() {
    let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
    let mesh = mesh(
        vec![
            vertex(Vec3::new(-1.0, -1.0, 0.0)),
            vertex(Vec3::new(1.0, -1.0, 0.0)),
            vertex(Vec3::new(0.0, 1.0, 0.0)),
            vertex(Vec3::new(-1.0, -1.0, 2.0)),
            vertex(Vec3::new(1.0, -1.0, 2.0)),
            vertex(Vec3::new(0.0, 1.0, 2.0)),
        ],
        vec![0, 1, 2, 3, 4, 5],
    );
    let hit = intersect_mesh(&ray, &mesh).unwrap();

    assert!((hit.distance - 1.0).abs() < TEST_EPSILON);
    assert_eq!(hit.triangle_index, 1);
}

#[test]
fn mesh_intersection_skips_invalid_indices() {
    let ray = Ray::new(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
    let mesh = mesh(
        vec![
            vertex(Vec3::new(-1.0, -1.0, 0.0)),
            vertex(Vec3::new(1.0, -1.0, 0.0)),
            vertex(Vec3::new(0.0, 1.0, 0.0)),
        ],
        vec![0, 99, 2, 0, 1, 2],
    );
    let hit = intersect_mesh(&ray, &mesh).unwrap();

    assert!((hit.distance - 1.0).abs() < TEST_EPSILON);
    assert_eq!(hit.triangle_index, 1);
}

#[test]
fn ray_hits_translated_mesh() {
    let ray = Ray::new(Vec3::new(5.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
    let mesh = triangle_mesh(0.0);
    let transform = Transform::new(Vec3::new(5.0, 0.0, 0.0), Quat::IDENTITY, Vec3::ONE);
    let hit = intersect_transformed_mesh(&ray, &mesh, &transform).unwrap();

    assert!((hit.distance - 1.0).abs() < TEST_EPSILON);
    assert!((hit.position - Vec3::new(5.0, 0.0, 0.0)).length() < TEST_EPSILON);
}

#[test]
fn ray_misses_translated_mesh() {
    let ray = Ray::new(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
    let mesh = triangle_mesh(0.0);
    let transform = Transform::new(Vec3::new(5.0, 0.0, 0.0), Quat::IDENTITY, Vec3::ONE);

    assert!(intersect_transformed_mesh(&ray, &mesh, &transform).is_none());
}

#[test]
fn ray_hits_scaled_mesh_with_world_distance() {
    let ray = Ray::new(Vec3::new(0.75, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
    let mesh = triangle_mesh(0.0);
    let transform = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::splat(2.0));
    let hit = intersect_transformed_mesh(&ray, &mesh, &transform).unwrap();

    assert!((hit.distance - 1.0).abs() < TEST_EPSILON);
    assert!((hit.position - Vec3::new(0.75, 0.0, 0.0)).length() < TEST_EPSILON);
}

#[test]
fn ray_hits_rotated_mesh() {
    let ray = Ray::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
    let mesh = triangle_mesh(0.0);
    let transform = Transform::new(Vec3::ZERO, Quat::from_rotation_y(FRAC_PI_2), Vec3::ONE);
    let hit = intersect_transformed_mesh(&ray, &mesh, &transform).unwrap();

    assert!((hit.distance - 1.0).abs() < TEST_EPSILON);
    assert!(hit.position.length() < TEST_EPSILON);
}

#[test]
fn ray_origin_inside_mesh_hits_forward_triangle() {
    let ray = Ray::new(Vec3::ZERO, Vec3::Z);
    let mesh = triangle_mesh(1.0);
    let hit = intersect_mesh(&ray, &mesh).unwrap();

    assert!((hit.distance - 1.0).abs() < TEST_EPSILON);
    assert!((hit.position - Vec3::new(0.0, 0.0, 1.0)).length() < TEST_EPSILON);
}
