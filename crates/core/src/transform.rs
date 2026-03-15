use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct Transform {
    pub position: Vec3,
    _padding1: f32,
    pub rotation: Quat,
    pub scale: Vec3,
    _padding2: f32,
}

impl Transform {
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Transform {
        Self {
            position,
            rotation,
            scale,
            ..Default::default()
        }
    }

    pub fn translate(&mut self, delta: Vec3) {
        self.position += delta;
    }

    pub fn rotate(&mut self, delta: Quat) {
        self.rotation = (self.rotation * delta).normalize();
    }
    pub fn scale(&mut self, delta: Vec3) {
        self.scale *= delta;
    }

    pub fn get_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.position)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_scale(self.scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    const EPSILON: f32 = 1e-6;

    fn assert_vec3_eq(a: Vec3, b: Vec3, msg: &str) {
        assert!(
            (a - b).length() < EPSILON,
            "{}: expected {:?}, got {:?}",
            msg,
            b,
            a
        );
    }

    fn assert_quat_eq(a: Quat, b: Quat, msg: &str) {
        // Quaternions q and -q represent the same rotation
        let dot = a.dot(b).abs();
        assert!(
            (dot - 1.0).abs() < EPSILON,
            "{}: expected {:?}, got {:?} (dot: {})",
            msg,
            b,
            a,
            dot
        );
    }

    #[test]
    fn test_identity_transform_produces_identity_matrix() {
        let transform = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);
        let matrix = transform.get_matrix();

        assert_eq!(
            matrix,
            Mat4::IDENTITY,
            "Identity transform should produce identity matrix"
        );
    }

    #[test]
    fn test_translation_only() {
        let position = Vec3::new(1.0, 2.0, 3.0);
        let transform = Transform::new(position, Quat::IDENTITY, Vec3::ONE);
        let matrix = transform.get_matrix();

        let expected = Mat4::from_translation(position);
        assert_eq!(
            matrix, expected,
            "Translation-only transform should match Mat4::from_translation"
        );

        // Verify translation component
        let translation = matrix.col(3).truncate();
        assert_vec3_eq(translation, position, "Matrix translation component");
    }

    #[test]
    fn test_rotation_only() {
        let rotation = Quat::from_rotation_y(PI / 2.0); // 90 degrees around Y
        let transform = Transform::new(Vec3::ZERO, rotation, Vec3::ONE);
        let matrix = transform.get_matrix();

        let expected = Mat4::from_quat(rotation);

        // Compare matrices element-wise with epsilon
        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (matrix.col(i)[j] - expected.col(i)[j]).abs() < EPSILON,
                    "Matrix mismatch at ({}, {}): expected {}, got {}",
                    i,
                    j,
                    expected.col(i)[j],
                    matrix.col(i)[j]
                );
            }
        }
    }

    #[test]
    fn test_scale_only() {
        let scale = Vec3::new(2.0, 3.0, 4.0);
        let transform = Transform::new(Vec3::ZERO, Quat::IDENTITY, scale);
        let matrix = transform.get_matrix();

        let expected = Mat4::from_scale(scale);
        assert_eq!(
            matrix, expected,
            "Scale-only transform should match Mat4::from_scale"
        );
    }

    #[test]
    fn test_combined_trs_matrix_order() {
        let position = Vec3::new(10.0, 0.0, 0.0);
        let rotation = Quat::from_rotation_y(PI / 2.0);
        let scale = Vec3::new(2.0, 2.0, 2.0);

        let transform = Transform::new(position, rotation, scale);
        let matrix = transform.get_matrix();

        // Manual TRS multiplication
        let expected =
            Mat4::from_translation(position) * Mat4::from_quat(rotation) * Mat4::from_scale(scale);

        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (matrix.col(i)[j] - expected.col(i)[j]).abs() < EPSILON,
                    "TRS matrix mismatch at ({}, {}): expected {}, got {}",
                    i,
                    j,
                    expected.col(i)[j],
                    matrix.col(i)[j]
                );
            }
        }
    }

    #[test]
    fn test_translate_updates_position() {
        let mut transform = Transform::new(Vec3::new(1.0, 2.0, 3.0), Quat::IDENTITY, Vec3::ONE);

        transform.translate(Vec3::new(0.5, -0.5, 1.0));

        assert_vec3_eq(
            transform.position,
            Vec3::new(1.5, 1.5, 4.0),
            "Position after translate",
        );
    }

    #[test]
    fn test_translate_multiple_times() {
        let mut transform = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);

        transform.translate(Vec3::new(1.0, 0.0, 0.0));
        transform.translate(Vec3::new(0.0, 1.0, 0.0));
        transform.translate(Vec3::new(0.0, 0.0, 1.0));

        assert_vec3_eq(
            transform.position,
            Vec3::new(1.0, 1.0, 1.0),
            "Position after multiple translates",
        );
    }

    #[test]
    fn test_rotation_updates_quaternion() {
        let mut transform = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);

        let delta_rotation = Quat::from_rotation_y(PI / 4.0); // 45 degrees
        transform.rotate(delta_rotation);

        assert_quat_eq(
            transform.rotation,
            delta_rotation,
            "Rotation after single update",
        );
    }

    #[test]
    fn test_rotation_accumulates() {
        let mut transform = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);

        // Apply 45 degrees twice = 90 degrees total
        let delta = Quat::from_rotation_y(PI / 4.0);
        transform.rotate(delta);
        transform.rotate(delta);

        let expected = Quat::from_rotation_y(PI / 2.0);
        assert_quat_eq(transform.rotation, expected, "Accumulated rotation");
    }

    #[test]
    fn test_rotation_stays_normalized() {
        let mut transform = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);

        // Apply many small rotations
        let delta = Quat::from_rotation_y(0.01);
        for _ in 0..1000 {
            transform.rotate(delta);
        }

        let length = transform.rotation.length();
        assert!(
            (length - 1.0).abs() < EPSILON,
            "Quaternion should stay normalized after many rotations, got length {}",
            length
        );
    }

    #[test]
    fn test_scale_multiplies_components() {
        let mut transform = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::new(2.0, 3.0, 4.0));

        transform.scale(Vec3::new(2.0, 0.5, 0.25));

        assert_vec3_eq(
            transform.scale,
            Vec3::new(4.0, 1.5, 1.0),
            "Scale after multiplication",
        );
    }

    #[test]
    fn test_scale_with_identity_no_change() {
        let mut transform = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::new(2.0, 3.0, 4.0));
        let original_scale = transform.scale;

        transform.scale(Vec3::ONE);

        assert_vec3_eq(
            transform.scale,
            original_scale,
            "Scale should not change with ONE",
        );
    }

    #[test]
    fn test_delta_time_translation() {
        let mut transform = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);

        let velocity = Vec3::new(10.0, 0.0, 0.0); // 10 units per second
        let delta_time = 0.1; // 100ms

        transform.translate(velocity * delta_time);

        assert_vec3_eq(
            transform.position,
            Vec3::new(1.0, 0.0, 0.0),
            "Position after delta time application",
        );
    }

    #[test]
    fn test_delta_time_rotation() {
        let mut transform = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);

        let angular_velocity = PI; // 180 degrees per second
        let delta_time = 0.5; // 500ms = half second

        let delta_rotation = Quat::from_rotation_y(angular_velocity * delta_time);
        transform.rotate(delta_rotation);

        // Should be 90 degrees
        let expected = Quat::from_rotation_y(PI / 2.0);
        assert_quat_eq(transform.rotation, expected, "Rotation after delta time");
    }

    #[test]
    fn test_delta_time_consistency() {
        // Two small steps should equal one large step
        let mut transform1 = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);
        let mut transform2 = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);

        let velocity = Vec3::new(5.0, 0.0, 0.0);

        // One large step
        transform1.translate(velocity * 0.1);

        // Two small steps
        transform2.translate(velocity * 0.05);
        transform2.translate(velocity * 0.05);

        assert_vec3_eq(
            transform1.position,
            transform2.position,
            "Delta time should be consistent across step sizes",
        );
    }

    #[test]
    fn test_transform_is_pod() {
        // Verify Transform can be safely transmitted to GPU
        let transform = Transform::new(
            Vec3::new(1.0, 2.0, 3.0),
            Quat::from_rotation_y(PI / 4.0),
            Vec3::new(2.0, 2.0, 2.0),
        );

        // Should be able to convert to bytes and back
        let bytes = bytemuck::bytes_of(&transform);
        let recovered: &Transform = bytemuck::from_bytes(bytes);

        assert_vec3_eq(
            recovered.position,
            transform.position,
            "Position after bytemuck roundtrip",
        );
        assert_vec3_eq(
            recovered.scale,
            transform.scale,
            "Scale after bytemuck roundtrip",
        );
        assert_quat_eq(
            recovered.rotation,
            transform.rotation,
            "Rotation after bytemuck roundtrip",
        );
    }

    #[test]
    fn test_matrix_transforms_point_correctly() {
        // Test that the matrix actually transforms points as expected
        let transform = Transform::new(
            Vec3::new(10.0, 0.0, 0.0),
            Quat::from_rotation_y(PI / 2.0), // 90 degrees
            Vec3::new(2.0, 2.0, 2.0),
        );

        let matrix = transform.get_matrix();
        let point = Vec3::new(1.0, 0.0, 0.0);
        let transformed = matrix.transform_point3(point);

        // Point (1,0,0) scaled by 2 = (2,0,0)
        // Rotated 90° around Y: (2,0,0) -> (0,0,-2)
        // Translated by (10,0,0): (0,0,-2) -> (10,0,-2)
        let expected = Vec3::new(10.0, 0.0, -2.0);

        assert_vec3_eq(transformed, expected, "Transformed point");
    }
}
