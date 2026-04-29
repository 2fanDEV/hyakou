use crate::{
    components::mesh_node::MeshNode,
    geometry::{ray::Ray, vertices::Vertex},
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
}

pub fn intersect_mesh(ray: &Ray, node: &MeshNode) -> Option<TriangleHit> {
    let mut closest = None;
    let vertices = &node.vertices;

    for (triangle_index, triangle) in node.indices.chunks_exact(3).enumerate() {
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
            });
        }
    }
    closest
}

#[cfg(test)]
#[path = "math_tests.rs"]
mod tests;
