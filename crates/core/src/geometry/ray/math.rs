use crate::{
    geometry::{mesh::Mesh, ray::Ray, vertices::Vertex},
    types::transform::Transform,
};

const EPSILON: f32 = 0.000001;

#[cfg(test)]
pub(super) const TEST_EPSILON: f32 = EPSILON;

pub fn intersect_triangle(ray: &Ray, v0: &Vertex, v1: &Vertex, v2: &Vertex) -> Option<f32> {
    let edge1 = v1.position - v0.position;
    let edge2 = v2.position - v0.position;
    let p = ray.direction().cross(edge2);
    let determinant = edge1.dot(p);

    if determinant.abs() < EPSILON {
        return None;
    }

    let inverse_determinant = determinant.recip();
    let origin_to_v0 = ray.origin() - v0.position;
    let u = origin_to_v0.dot(p) * inverse_determinant;

    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = origin_to_v0.cross(edge1);
    let v = ray.direction().dot(q) * inverse_determinant;

    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let distance = edge2.dot(q) * inverse_determinant;

    (distance >= 0.0).then_some(distance)
}

pub struct TriangleHit {
    pub distance: f32,
    pub triangle_index: usize,
    pub position: glam::Vec3,
}

pub fn intersect_mesh(ray: &Ray, mesh: &Mesh) -> Option<TriangleHit> {
    let mut closest = None;
    let vertices = &mesh.vertices;

    for (triangle_index, triangle) in mesh.indices.chunks_exact(3).enumerate() {
        let (Some(v0), Some(v1), Some(v2)) = (
            vertices.get(triangle[0] as usize),
            vertices.get(triangle[1] as usize),
            vertices.get(triangle[2] as usize),
        ) else {
            continue;
        };

        let Some(distance) = intersect_triangle(ray, v0, v1, v2) else {
            continue;
        };

        if closest
            .as_ref()
            .is_none_or(|hit: &TriangleHit| distance < hit.distance)
        {
            closest = Some(TriangleHit {
                distance,
                triangle_index,
                position: ray.origin() + ray.direction() * distance,
            });
        }
    }
    closest
}

pub fn intersect_transformed_mesh(
    ray: &Ray,
    mesh: &Mesh,
    transform: &Transform,
) -> Option<TriangleHit> {
    let model = transform.get_matrix();
    let inverse_model = model.inverse();
    let local_origin = inverse_model.transform_point3(ray.origin());
    let local_direction = inverse_model.transform_vector3(ray.direction()).normalize();

    if !local_origin.is_finite() || !local_direction.is_finite() {
        return None;
    }

    let local_ray = Ray::new(local_origin, local_direction);
    let local_hit = intersect_mesh(&local_ray, mesh)?;
    let world_position = model.transform_point3(local_hit.position);

    if !world_position.is_finite() {
        return None;
    }

    Some(TriangleHit {
        distance: world_position.distance(ray.origin()),
        triangle_index: local_hit.triangle_index,
        position: world_position,
    })
}

#[cfg(test)]
#[path = "math_tests.rs"]
mod tests;
