use anyhow::Ok;

use crate::renderer::{
    components::transform::{self},
    trajectory::Trajectory,
};

pub struct StationaryTrajectory {}

impl Trajectory for StationaryTrajectory {
    fn animate(
        &mut self,
        t: Option<&transform::Transform>,
        delta: crate::renderer::types::DeltaTime,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn reset(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stationary_trajectory_creation() {
        let _trajectory = StationaryTrajectory {};
        // Just verify it can be created
    }

    #[test]
    fn test_stationary_trajectory_animate_returns_ok() {
        let mut trajectory = StationaryTrajectory {};
        let result = trajectory.animate(None, 1.0);
        assert!(result.is_ok());

        // Also test with a target transform
        let target = transform::Transform::default();
        let result = trajectory.animate(Some(&target), 0.5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_stationary_trajectory_reset_does_nothing() {
        let mut trajectory = StationaryTrajectory {};

        // Reset should not panic or fail
        trajectory.reset();

        // Should still be able to animate after reset
        let result = trajectory.animate(None, 1.0);
        assert!(result.is_ok());
    }
}
