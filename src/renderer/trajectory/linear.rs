use std::sync::{Arc, RwLock};

use anyhow::anyhow;
use glam::Vec3;
use log::debug;

use crate::renderer::{
    components::transform::Transform,
    trajectory::{Direction, Trajectory, calculate_direction_vector},
};

pub struct LinearTrajectory {
    transform: Arc<RwLock<Transform>>,
    start_position: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
    speed: f32,
    progress: f32,
    looping: bool,
    direction: Direction,
}

impl LinearTrajectory {
    pub fn new(
        transform: Arc<RwLock<Transform>>,
        start_position: Vec3,
        yaw: f32,
        pitch: f32,
        distance: f32,
        speed: f32,
        looping: bool,
    ) -> Self {
        Self {
            transform,
            start_position,
            yaw,
            pitch,
            distance,
            speed,
            progress: 0.0,
            looping,
            direction: Direction::FORWARDS,
        }
    }
}

impl Trajectory for LinearTrajectory {
    fn animate(
        &mut self,
        _target: Option<&Transform>,
        delta: crate::renderer::types::DeltaTime,
    ) -> anyhow::Result<()> {
        if let Some(mut transform) = self.transform.try_write().ok() {
            let direction_vector = calculate_direction_vector(self.yaw, self.pitch);
            match self.direction {
                Direction::FORWARDS => {
                    self.progress += (self.speed / self.distance) * delta;
                }
                Direction::BACKWARDS => {
                    self.progress -= (self.speed / self.distance) * delta;
                }
                _ => {
                    return Err(anyhow!(
                        "Only forwards or backwards movement allowed in linear trajectory"
                    ));
                }
            }
            self.progress = self.progress.clamp(-1.0, 1.0);
            transform.position =
                self.start_position + direction_vector * self.distance * self.progress;

            if self.progress >= 1.0 {
                self.direction = Direction::BACKWARDS;
            }
            if self.progress <= -1.0 {
                self.direction = Direction::FORWARDS;
            }
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.progress = 0.0;
        self.direction = Direction::FORWARDS;
        if let Some(mut transform) = self.transform.try_write().ok() {
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
        let mut trajectory = LinearTrajectory::new(
            transform.clone(),
            start_pos,
            0.0,   // yaw: move along X axis
            0.0,   // pitch: no vertical component
            10.0,  // distance: 10 units
            5.0,   // speed: 5 units per second
            false, // not looping
        );

        // Simulate 1 second of movement
        trajectory.animate(None, 1.0).unwrap();

        // After 1 second at 5 units/sec over 10 units distance, progress should be 0.5
        assert!((trajectory.progress - 0.5).abs() < 0.001);

        // Position should be halfway along the path
        let pos = transform.read().unwrap().position;
        assert!((pos.x - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_linear_trajectory_bounce_at_boundaries() {
        let transform = Arc::new(RwLock::new(Transform::default()));
        let start_pos = Vec3::new(0.0, 0.0, 0.0);
        let mut trajectory = LinearTrajectory::new(
            transform.clone(),
            start_pos,
            90.0, // yaw: move along Y axis
            0.0,  // pitch
            4.0,  // distance: 4 units
            2.0,  // speed: 2 units per second
            true, // looping enabled
        );

        // Animate forward to boundary (2 seconds to reach end)
        trajectory.animate(None, 3.5).unwrap();

        // Should have bounced back and changed direction
        assert_eq!(trajectory.direction, Direction::BACKWARDS);
        assert!(trajectory.progress <= 1.0, "{:?}", trajectory.progress);

        // Continue backward past start
        trajectory.animate(None, 5.0).unwrap();

        // Should bounce forward again
        assert_eq!(trajectory.direction, Direction::FORWARDS);
        assert!(trajectory.progress >= -1.0);
    }

    #[test]
    fn test_linear_trajectory_reset() {
        let transform = Arc::new(RwLock::new(Transform::default()));
        let start_pos = Vec3::new(5.0, 10.0, -3.0);
        let mut trajectory =
            LinearTrajectory::new(transform.clone(), start_pos, 45.0, 0.0, 8.0, 4.0, true);

        // Move the trajectory
        trajectory.animate(None, 1.0).unwrap();
        assert!(trajectory.progress > 0.0);

        // Reset
        trajectory.reset();

        // Check state is reset
        assert_eq!(trajectory.progress, 0.0);
        assert_eq!(trajectory.direction, Direction::FORWARDS);

        let pos = transform.read().unwrap().position;
        assert_eq!(pos, start_pos);
    }
}
