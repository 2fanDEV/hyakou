use std::sync::{Arc, RwLock};

use anyhow::{Result, anyhow};

use crate::renderer::{components::transform::Transform, trajectory::Trajectory, types::DeltaTime};

#[derive(Default, Clone)]
pub struct CircularTrajectory {
    target_transform: Arc<RwLock<Transform>>,
    radius: f32,
    angle: f32,
    speed: f32,
}

impl CircularTrajectory {
    pub fn new(transform: Arc<RwLock<Transform>>, radius: f32) -> Self {
        Self {
            target_transform: transform,
            radius,
            speed: 100f32,
            ..Default::default()
        }
    }
}

impl Trajectory for CircularTrajectory {
    fn animate(&mut self, target: Option<&Transform>, delta: DeltaTime) -> Result<()> {
        if let Some(mut transform) = self.target_transform.try_write().ok() {
            if let Some(t) = target {
                transform.position.x =
                    t.position.x + self.radius * f32::cos(self.angle.to_radians());
                transform.position.z =
                    t.position.z + self.radius * f32::sin(self.angle.to_radians());
            } else {
                transform.position.x = self.radius * f32::cos(self.angle.to_radians());
                transform.position.z = self.radius * f32::sin(self.angle.to_radians());
            }
            self.angle += self.speed * delta;
            Ok(())
        } else {
            return Err(anyhow!(
                "Failed to acquire lock on mesh transform {:?}",
                self.target_transform
            ));
        }
    }

    fn reset(&mut self) {
        self.angle = 0f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_circular_trajectory_rotation_without_target() {
        let transform = Arc::new(RwLock::new(Transform::default()));
        let radius = 5.0;
        let mut trajectory = CircularTrajectory::new(transform.clone(), radius);

        // Initial position should be at angle 0 (radius units along X axis)
        trajectory.animate(None, 0.0).unwrap();
        let pos = transform.read().unwrap().position;
        assert!((pos.x - radius).abs() < 0.001);
        assert!(pos.z.abs() < 0.001);

        // After rotation, angle should have increased
        let initial_angle = trajectory.angle;
        trajectory.animate(None, 0.1).unwrap();
        assert!(trajectory.angle > initial_angle);

        // Position should still be on the circle (distance from origin = radius)
        let pos = transform.read().unwrap().position;
        let distance_from_origin = (pos.x * pos.x + pos.z * pos.z).sqrt();
        assert!((distance_from_origin - radius).abs() < 0.001);
    }

    #[test]
    fn test_circular_trajectory_rotation_with_target() {
        let transform = Arc::new(RwLock::new(Transform::default()));
        let mut target_transform = Transform::default();
        target_transform.position = Vec3::new(10.0, 0.0, 20.0);

        let radius = 3.0;
        let mut trajectory = CircularTrajectory::new(transform.clone(), radius);

        // Animate around target
        trajectory.animate(Some(&target_transform), 0.0).unwrap();

        // Position should be offset from target by radius
        let pos = transform.read().unwrap().position;
        let offset_x = pos.x - target_transform.position.x;
        let offset_z = pos.z - target_transform.position.z;
        let distance_from_target = (offset_x * offset_x + offset_z * offset_z).sqrt();

        assert!((distance_from_target - radius).abs() < 0.001);
        assert!((pos.y - target_transform.position.y).abs() < 0.001); // Y should match target
    }

    #[test]
    fn test_circular_trajectory_reset() {
        let transform = Arc::new(RwLock::new(Transform::default()));
        let mut trajectory = CircularTrajectory::new(transform.clone(), 7.0);

        // Rotate for some time
        trajectory.animate(None, 1.0).unwrap();
        assert!(trajectory.angle > 0.0);

        // Reset
        trajectory.reset();

        // Angle should be back to 0
        assert_eq!(trajectory.angle, 0.0);
    }
}
