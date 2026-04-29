use glam::Vec3;

use super::{TEST_EPSILON, intersect_mesh, intersect_triangle};
use crate::{
    components::mesh_node::MeshNode,
    geometry::{mesh::Mesh, node::NodeMetadata, ray::Ray, vertices::Vertex},
    types::transform::Transform,
};

fn vertex(position: Vec3) -> Vertex {
    Vertex {
        position,
        ..Default::default()
    }
}

fn mesh_node(vertices: Vec<Vertex>, indices: Vec<u32>) -> MeshNode {
    MeshNode::new(
        Mesh::new(None, None, vertices, indices),
        Transform::default(),
        NodeMetadata::default(),
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
    let node = mesh_node(
        vec![
            vertex(Vec3::new(-1.0, -1.0, 0.0)),
            vertex(Vec3::new(1.0, -1.0, 0.0)),
            vertex(Vec3::new(0.0, 1.0, 0.0)),
        ],
        vec![0, 1, 2],
    );
    let hit = intersect_mesh(&ray, &node).unwrap();

    assert!((hit.distance - 1.0).abs() < TEST_EPSILON);
    assert_eq!(hit.triangle_index, 0);
}

#[test]
fn ray_misses_mesh() {
    let ray = Ray::new(Vec3::new(2.0, 2.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
    let node = mesh_node(
        vec![
            vertex(Vec3::new(-1.0, -1.0, 0.0)),
            vertex(Vec3::new(1.0, -1.0, 0.0)),
            vertex(Vec3::new(0.0, 1.0, 0.0)),
        ],
        vec![0, 1, 2],
    );

    assert!(intersect_mesh(&ray, &node).is_none());
}

#[test]
fn ray_hits_closest_mesh_triangle() {
    let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
    let node = mesh_node(
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
    let hit = intersect_mesh(&ray, &node).unwrap();

    assert!((hit.distance - 1.0).abs() < TEST_EPSILON);
    assert_eq!(hit.triangle_index, 1);
}

#[test]
fn mesh_intersection_skips_invalid_indices() {
    let ray = Ray::new(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
    let node = mesh_node(
        vec![
            vertex(Vec3::new(-1.0, -1.0, 0.0)),
            vertex(Vec3::new(1.0, -1.0, 0.0)),
            vertex(Vec3::new(0.0, 1.0, 0.0)),
        ],
        vec![0, 99, 2, 0, 1, 2],
    );
    let hit = intersect_mesh(&ray, &node).unwrap();

    assert!((hit.distance - 1.0).abs() < TEST_EPSILON);
    assert_eq!(hit.triangle_index, 1);
}
