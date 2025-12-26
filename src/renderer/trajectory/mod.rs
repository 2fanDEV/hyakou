use anyhow::Result;
use glam::Vec3;

use crate::renderer::{components::transform::Transform, types::DeltaTime};

pub mod circular;
pub mod linear;
pub mod stationary;

/// A trait to implement when specific trajectory path are to be implemented.
/// The animate(...) most likely uses a try_wtite on a Arc<RwLock<Transform>> which could
/// panic but should be handled gracefully. Nonetheless you can match the result to get the
/// error that occurs when try_write fails to acquire the lock.
pub trait Trajectory {
    fn animate(&mut self, t: Option<&Transform>, delta: DeltaTime) -> Result<()>;
    fn reset(&mut self);
}

pub fn calculate_direction_vector(yaw_radians: f32, pitch_radians: f32) -> Vec3 {
    let x = pitch_radians.cos() * yaw_radians.cos();
    let y = pitch_radians.cos() * yaw_radians.sin();
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
    fn test_direction_vector_zero_angles() {
        let dir = calculate_direction_vector(0.0, 0.0);
        assert!((dir.x - 1.0).abs() < 0.001);
        assert!(dir.y.abs() < 0.001);
        assert!(dir.z.abs() < 0.001);
    }

    #[test]
    fn test_direction_vector_90_yaw() {
        let dir = calculate_direction_vector(90.0, 0.0);
        assert!(dir.x.abs() < 0.001);
        assert!((dir.y - 1.0).abs() < 0.001);
        assert!(dir.z.abs() < 0.001);
    }

    #[test]
    fn test_direction_vector_90_pitch() {
        let dir = calculate_direction_vector(0.0, 90.0);
        assert!(dir.x.abs() < 0.001);
        assert!(dir.y.abs() < 0.001);
        assert!((dir.z - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_direction_vector_90_pitch_90_yaw() {
        let dir = calculate_direction_vector(90.0, 90.0);
        assert!(dir.x.abs() < 0.21, "x: {:?} < 0.21", dir.x.abs());
        assert!(dir.y.abs() < 0.00000005, "y: {:?} < 0.004", dir.y.abs());
        assert!(
            dir.z.abs() - 1.0 < 0.001,
            "z: {:?} - 1.0 < 0.001",
            dir.z.abs()
        );
    }
}
