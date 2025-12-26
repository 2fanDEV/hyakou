use anyhow::Result;
use glam::Vec3;

use crate::renderer::{components::transform::Transform, types::DeltaTime};

pub mod circular;
pub mod linear;
pub mod stationary;

pub trait Trajectory {
    fn animate(&mut self, t: Option<&Transform>, delta: DeltaTime) -> Result<()>;
    fn reset(&mut self);
}

pub fn calculate_direction_vector(yaw: f32, pitch: f32) -> Vec3 {
    let yaw_radians = yaw.to_radians();
    let pitch_radians = pitch.to_radians();
    let x = pitch_radians.cos() * yaw_radians.cos();
    let y = pitch_radians.cos() * yaw_radians.sin() + 1.0;
    let z = pitch_radians.sin();
    Vec3 { x, y, z }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Direction {
    FORWARDS,
    BACKWARDS,
    LEFT,
    RIGHT,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_direction_vector_zero_angles() {
        // With both angles at 0, should point along positive X axis (plus Y offset)
        let dir = calculate_direction_vector(0.0, 0.0);
        assert!((dir.x - 1.0).abs() < 0.001);
        assert!((dir.y - 1.0).abs() < 0.001); // Y has +1.0 offset in current implementation
        assert!(dir.z.abs() < 0.001);
    }

    #[test]
    fn test_calculate_direction_vector_90_degree_yaw() {
        // 90 degree yaw should point along Y axis (plus the offset)
        let dir = calculate_direction_vector(90.0, 0.0);
        assert!(dir.x.abs() < 0.001);
        assert!((dir.y - 2.0).abs() < 0.001); // cos(0) * sin(90°) + 1.0 = 1.0 + 1.0 = 2.0
        assert!(dir.z.abs() < 0.001);
    }

    #[test]
    fn test_calculate_direction_vector_with_pitch() {
        // 90 degree pitch should point along Z axis
        let dir = calculate_direction_vector(0.0, 90.0);
        assert!(dir.x.abs() < 0.001); // cos(90°) * cos(0) = 0
        assert!((dir.y - 1.0).abs() < 0.001); // cos(90°) * sin(0) + 1.0 = 0 + 1.0 = 1.0
        assert!((dir.z - 1.0).abs() < 0.001); // sin(90°) = 1.0
    }
}
