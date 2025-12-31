use anyhow::Ok;

use crate::renderer::{
    animator::Animation,
    components::transform::{self},
    types::ids::MeshId,
};

pub struct StationaryTrajectory {
    pub id: MeshId,
}

impl Animation for StationaryTrajectory {
    fn animate(
        &mut self,
        _t: Option<&transform::Transform>,
        _delta: crate::renderer::types::DeltaTime,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    // Empty as you don't need this to do anything as it's stationary.
    fn reset(&mut self) {}

    fn get_id(&self) -> &MeshId {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn test_stationary_trajectory_creation() {
        let _trajectory = StationaryTrajectory {
            id: MeshId(Uuid::new_v4().to_string()),
        };
        // Just verify it can be created
    }

    #[test]
    fn test_stationary_trajectory_animate_returns_ok() {
        let mut trajectory = StationaryTrajectory {
            id: MeshId(Uuid::new_v4().to_string()),
        };
        let result = trajectory.animate(None, 1.0);
        assert!(result.is_ok());

        // Also test with a target transform
        let target = transform::Transform::default();
        let result = trajectory.animate(Some(&target), 0.5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_stationary_trajectory_reset_does_nothing() {
        let mut trajectory = StationaryTrajectory {
            id: MeshId(Uuid::new_v4().to_string()),
        };

        // Reset should not panic or fail
        trajectory.reset();

        // Should still be able to animate after reset
        let result = trajectory.animate(None, 1.0);
        assert!(result.is_ok());
    }
}
