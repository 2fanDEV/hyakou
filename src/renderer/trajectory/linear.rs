use parking_lot::RwLock;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use glam::Vec3;

use crate::renderer::{
    components::{render_mesh::RenderMesh, transform::Transform},
    trajectory::{Direction, Trajectory, calculate_direction_vector},
};

/// LinearTrajectory is an animation that allows the
/// objects transformation to move a linear path. Back/Forwards. Diagonally an so on and so forth
/// Yaw, Pitch are supposed to be passed in as radians.
/// Speed is in units/second, distance is in units
///
#[derive(Debug, Clone)]
pub struct LinearTrajectory {
    pub id: String,
    transform: Arc<RwLock<Transform>>,
    start_position: Vec3,
    yaw_radians: f32,
    /// in radians
    pitch_radians: f32,
    /// in radians
    distance: f32,
    /// in Units
    speed: f32,
    /// in units/second
    progress: f32,
    /// between -1.0 and 1.0
    looping: bool,
    reversing: bool,
    direction: Direction,
}

impl LinearTrajectory {
    const MIN_PROGRESS: f32 = -1.0;
    const MAX_PROGRESS: f32 = 1.0;
    const ZERO_PROGRESS: f32 = 0.0;

    pub fn new_deconstructed_mesh(
        id: String,
        transform: Arc<RwLock<Transform>>,
        start_position: Vec3,
        yaw_radians: f32,
        pitch_radians: f32,
        distance: f32,
        speed: f32,
        looping: bool,
        reversing: bool,
    ) -> Result<Self> {
        if distance == 0.0 || speed == 0.0 {
            return Err(anyhow!("Distance and speed must be non-zero!"));
        }
        Ok(Self {
            id,
            transform,
            start_position,
            yaw_radians,
            pitch_radians,
            distance,
            speed,
            progress: Self::ZERO_PROGRESS,
            looping,
            reversing,
            direction: Direction::FORWARDS,
        })
    }

    pub fn new(
        render_mesh: RenderMesh,
        start_position: Vec3,
        yaw_radians: f32,
        pitch_radians: f32,
        distance: f32,
        speed: f32,
        looping: bool,
        reversing: bool,
    ) -> Result<Self> {
        Self::new_deconstructed_mesh(
            render_mesh.id,
            render_mesh.transform,
            start_position,
            yaw_radians,
            pitch_radians,
            distance,
            speed,
            looping,
            reversing,
        )
    }
}

impl Trajectory for LinearTrajectory {
    /// Currently ignoring target since there is no use for it yet.
    /// Maybe sometime in the future this will cause the linear trajectory to be right above the target
    fn animate(
        &mut self,
        _target: Option<&Transform>,
        delta: crate::renderer::types::DeltaTime,
    ) -> anyhow::Result<()> {
        if let Some(mut transform) = self.transform.try_write() {
            let direction_vector = calculate_direction_vector(self.yaw_radians, self.pitch_radians);
            match self.direction {
                Direction::FORWARDS => {
                    self.progress += (self.speed / self.distance) * delta;
                }
                Direction::BACKWARDS => {
                    self.progress -= (self.speed / self.distance) * delta;
                }
            }
            self.progress = self.progress.clamp(Self::MIN_PROGRESS, Self::MAX_PROGRESS);
            transform.position =
                self.start_position + direction_vector * self.distance * self.progress;

            if self.progress >= Self::MAX_PROGRESS && self.reversing {
                self.direction = Direction::BACKWARDS;
            }
            if self.progress <= Self::MIN_PROGRESS && self.looping {
                self.direction = Direction::FORWARDS;
            }

            if self.progress >= Self::MAX_PROGRESS && self.looping && !self.reversing {
                transform.position = self.start_position;
                self.progress = Self::ZERO_PROGRESS;
            }
        } else {
            return Err(anyhow!(
                "Failed to acquire lock on transform: {:?}",
                self.id
            ));
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.progress = Self::ZERO_PROGRESS;
        self.direction = Direction::FORWARDS;
        if let Some(mut transform) = self.transform.try_write() {
            transform.position = self.start_position;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_trajectory_forward_movement() {
        let transform = Arc::new(RwLock::new(Transform::default()));
        let start_pos = Vec3::new(0.0, 0.0, 0.0);
        let mut trajectory = LinearTrajectory::new_deconstructed_mesh(
            "Test".to_string(),
            transform.clone(),
            start_pos,
            0.0,   // yaw: move along X axis
            0.0,   // pitch: no vertical component
            10.0,  // distance: 10 units
            5.0,   // speed: 5 units per second
            false, // not looping,
            true,
        )
        .unwrap();

        // Simulate 1 second of movement
        trajectory.animate(None, 1.0).unwrap();

        // After 1 second at 5 units/sec over 10 units distance, progress should be 0.5
        assert!((trajectory.progress - 0.5).abs() < 0.001);

        // Position should be halfway along the path
        let pos = transform.read().position;
        assert!((pos.x - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_linear_trajectory_bounce_at_boundaries() {
        let transform = Arc::new(RwLock::new(Transform::default()));
        let start_pos = Vec3::new(0.0, 0.0, 0.0);
        let mut trajectory = LinearTrajectory::new_deconstructed_mesh(
            "Test1".to_string(),
            transform.clone(),
            start_pos,
            f32::to_radians(90.0), // yaw: move along Y axis
            f32::to_radians(0.0),  // pitch
            4.0,                   // distance: 4 units
            2.0,                   // speed: 2 units per second
            true,                  // looping enabled
            true,
        )
        .unwrap();

        // Animate forward to boundary (2 seconds to reach end)
        trajectory.animate(None, 3.5).unwrap();

        // Should have bounced back and changed direction
        assert_eq!(trajectory.direction, Direction::BACKWARDS);
        assert!(
            trajectory.progress <= LinearTrajectory::MAX_PROGRESS,
            "{:?}",
            trajectory.progress
        );

        // Continue backward past start
        trajectory.animate(None, 5.0).unwrap();

        // Should bounce forward again
        assert_eq!(trajectory.direction, Direction::FORWARDS);
        assert!(trajectory.progress >= LinearTrajectory::MIN_PROGRESS);
    }

    #[test]
    fn test_linear_trajectory_reset() {
        let transform = Arc::new(RwLock::new(Transform::default()));
        let start_pos = Vec3::new(5.0, 10.0, -3.0);
        let mut trajectory = LinearTrajectory::new_deconstructed_mesh(
            "Test".to_string(),
            transform.clone(),
            start_pos,
            45.0,
            0.0,
            8.0,
            4.0,
            true,
            true,
        )
        .unwrap();

        // Move the trajectory
        trajectory.animate(None, 1.0).unwrap();
        assert!(trajectory.progress > LinearTrajectory::ZERO_PROGRESS);

        // Reset
        trajectory.reset();

        // Check state is reset
        assert_eq!(trajectory.progress, LinearTrajectory::ZERO_PROGRESS);
        assert_eq!(trajectory.direction, Direction::FORWARDS);

        let pos = transform.read().position;
        assert_eq!(pos, start_pos);
    }
}
