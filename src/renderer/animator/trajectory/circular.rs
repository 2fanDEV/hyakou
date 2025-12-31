use parking_lot::RwLock;
use std::{f32::consts::PI, sync::Arc};

use anyhow::{Result, anyhow};

use crate::renderer::{
    animator::Animation,
    components::{render_mesh::RenderMesh, transform::Transform},
    types::{DeltaTime, ids::MeshId},
};

#[derive(Default, Clone)]
pub struct CircularTrajectory {
    id: MeshId,
    transform: Arc<RwLock<Transform>>,
    radius: f32,
    angle: f32,
    speed: f32,
}

impl CircularTrajectory {
    pub fn new_deconstructed_mesh(
        id: MeshId,
        transform: Arc<RwLock<Transform>>,
        radius: f32,
        speed: f32,
    ) -> Result<Self> {
        if radius <= 0.0 || speed <= 0.0 {
            return Err(anyhow!(
                "Radius and speed have to be non-negative/non-zero!"
            ));
        }
        Ok(Self {
            id,
            transform,
            radius,
            speed,
            angle: 0.0,
        })
    }

    pub fn new(render_mesh: RenderMesh, radius: f32, speed: f32) -> Result<Self> {
        Self::new_deconstructed_mesh(render_mesh.id, render_mesh.transform, radius, speed)
    }
}

impl Animation for CircularTrajectory {
    fn animate(&mut self, target: Option<&Transform>, delta: DeltaTime) -> Result<()> {
        if let Some(mut transform) = self.transform.try_write() {
            if let Some(t) = target {
                transform.position.x = t.position.x + self.radius * f32::cos(self.angle);
                transform.position.y = t.position.y;
                transform.position.z = t.position.z + self.radius * f32::sin(self.angle);
            } else {
                transform.position.x = self.radius * f32::cos(self.angle);
                transform.position.z = self.radius * f32::sin(self.angle);
            }
            self.angle = (self.angle + self.speed * delta) % (2.0 * PI);
            Ok(())
        } else {
            return Err(anyhow!(
                "Failed to acquire lock on mesh transform {:?}",
                self.id
            ));
        }
    }

    fn reset(&mut self) {
        self.angle = 0f32;
    }

    fn get_id(&self) -> &MeshId {
        &self.id
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
        let mut trajectory = CircularTrajectory::new_deconstructed_mesh(
            MeshId("TEST".to_string()),
            transform.clone(),
            radius,
            100f32,
        )
        .unwrap();

        // Initial position should be at angle 0 (radius units along X axis)
        trajectory.animate(None, 0.0).unwrap();
        let pos = transform.read().position;
        assert!((pos.x - radius).abs() < 0.001);
        assert!(pos.z.abs() < 0.001);

        // After rotation, angle should have increased
        let initial_angle = trajectory.angle;
        trajectory.animate(None, 0.1).unwrap();
        assert!(trajectory.angle > initial_angle);

        // Position should still be on the circle (distance from origin = radius)
        let pos = transform.read().position;
        let distance_from_origin = (pos.x * pos.x + pos.z * pos.z).sqrt();
        assert!((distance_from_origin - radius).abs() < 0.001);
    }

    #[test]
    fn test_circular_trajectory_rotation_with_target() {
        let transform = Arc::new(RwLock::new(Transform::default()));
        let mut target_transform = Transform::default();
        target_transform.position = Vec3::new(10.0, 0.0, 20.0);

        let radius = 3.0;
        let mut trajectory = CircularTrajectory::new_deconstructed_mesh(
            MeshId("TEST".to_string()),
            transform.clone(),
            radius,
            100f32,
        )
        .unwrap();

        // Animate around target
        trajectory.animate(Some(&target_transform), 0.0).unwrap();

        // Position should be offset from target by radius
        let pos = transform.read().position;
        let offset_x = pos.x - target_transform.position.x;
        let offset_z = pos.z - target_transform.position.z;
        let distance_from_target = (offset_x * offset_x + offset_z * offset_z).sqrt();

        assert!((distance_from_target - radius).abs() < 0.001);
        assert!((pos.y - target_transform.position.y).abs() < 0.001); // Y should match target
    }

    #[test]
    fn test_circular_trajectory_reset() {
        let transform = Arc::new(RwLock::new(Transform::default()));
        let mut trajectory = CircularTrajectory::new_deconstructed_mesh(
            MeshId("TEST".to_string()),
            transform.clone(),
            7.0,
            100f32,
        )
        .unwrap();

        // Rotate for some time
        trajectory.animate(None, 1.0).unwrap();
        assert!(trajectory.angle > 0.0);

        // Reset
        trajectory.reset();

        // Angle should be back to 0
        assert_eq!(trajectory.angle, 0.0);
    }
}
