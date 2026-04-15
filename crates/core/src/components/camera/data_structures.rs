use glam::Vec3;

use crate::types::{DeltaTime, shared::Coordinates3};

#[derive(Debug)]
pub enum CameraMode {
    FLY,
    PAN,
    ORBIT,
}

#[derive(Debug)]
pub struct CameraAxes {
    pub forward: Vec3,
    pub forward_mag: f32,
    pub right: Vec3,
    pub view_up: Vec3,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CameraAnimationEasing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl CameraAnimationEasing {
    pub fn apply(self, progress: f32) -> f32 {
        let progress = progress.clamp(0.0, 1.0);

        match self {
            Self::Linear => progress,
            Self::EaseIn => progress * progress,
            Self::EaseOut => 1.0 - (1.0 - progress).powi(2),
            Self::EaseInOut => match progress {
                value if value < 0.5 => 2.0 * value * value,
                value => 1.0 - (-2.0 * value + 2.0).powi(2) / 2.0,
            },
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::EaseIn => "ease-in",
            Self::EaseOut => "ease-out",
            Self::EaseInOut => "ease-in-out",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraAnimationRequest {
    target_coords: Coordinates3,
    duration_seconds: Option<f32>,
    easing: CameraAnimationEasing,
}

impl CameraAnimationRequest {
    const DEFAULT_DURATION_SECONDS: f32 = 1.0;
    const MIN_DISTANCE: f32 = 0.001;
    const MIN_DURATION_SECONDS: f32 = 0.001;

    pub fn new(
        target_coords: Coordinates3,
        duration_seconds: Option<f32>,
        easing: CameraAnimationEasing,
    ) -> Self {
        Self {
            target_coords,
            duration_seconds,
            easing,
        }
    }

    pub fn from_target(target_coords: Coordinates3) -> Self {
        Self::new(target_coords, None, CameraAnimationEasing::default())
    }

    pub fn easing(self) -> CameraAnimationEasing {
        self.easing
    }

    pub fn resolve_duration_seconds(
        self,
        start_coords: Coordinates3,
        speed_units_per_second: f32,
    ) -> f32 {
        match self.duration_seconds {
            Some(duration_seconds) => duration_seconds.max(Self::MIN_DURATION_SECONDS),
            None => {
                let distance = start_coords.to_vec().distance(self.target_coords.to_vec());
                if distance <= Self::MIN_DISTANCE {
                    return Self::MIN_DURATION_SECONDS;
                }

                if speed_units_per_second.is_finite() && speed_units_per_second > 0.0 {
                    (distance / speed_units_per_second).max(Self::MIN_DURATION_SECONDS)
                } else {
                    Self::DEFAULT_DURATION_SECONDS
                }
            }
        }
    }

    pub fn target_coords(self) -> Coordinates3 {
        self.target_coords
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraAnimationStateSnapshot {
    pub active: bool,
    pub progress: f32,
    pub duration_seconds: f32,
    pub elapsed_seconds: f32,
    pub target_coords: Coordinates3,
    pub easing: CameraAnimationEasing,
}

impl CameraAnimationStateSnapshot {
    pub fn inactive(current_coords: Coordinates3) -> Self {
        Self {
            active: false,
            progress: 0.0,
            duration_seconds: 0.0,
            elapsed_seconds: 0.0,
            target_coords: current_coords,
            easing: CameraAnimationEasing::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraTransition {
    start_coords: Coordinates3,
    target_coords: Coordinates3,
    duration_seconds: f32,
    elapsed_seconds: f32,
    status: TransitionStatus,
    easing: CameraAnimationEasing,
}

impl CameraTransition {
    const COMPLETE_DISTANCE: f32 = 0.001;

    pub fn new(
        start_coords: Coordinates3,
        request: CameraAnimationRequest,
        speed_units_per_second: f32,
    ) -> Self {
        let duration_seconds =
            request.resolve_duration_seconds(start_coords, speed_units_per_second);
        let mut transition = Self {
            start_coords,
            target_coords: request.target_coords(),
            duration_seconds,
            elapsed_seconds: 0.0,
            status: TransitionStatus::Active,
            easing: request.easing(),
        };

        if transition.is_at_target() {
            transition.complete();
        }

        transition
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, TransitionStatus::Active)
    }

    pub fn target_coords(&self) -> Coordinates3 {
        self.target_coords
    }

    pub fn progress(&self) -> f32 {
        if self.duration_seconds <= 0.0 {
            1.0
        } else {
            (self.elapsed_seconds / self.duration_seconds).clamp(0.0, 1.0)
        }
    }

    pub fn duration_seconds(&self) -> f32 {
        self.duration_seconds
    }

    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed_seconds
    }

    pub fn advance(&mut self, delta_time: DeltaTime) -> Coordinates3 {
        if !self.is_active() {
            return self.current_position();
        }

        self.elapsed_seconds =
            (self.elapsed_seconds + delta_time.max(0.0)).min(self.duration_seconds);
        if self.progress() >= 1.0 {
            self.complete();
        }

        self.current_position()
    }

    pub fn stop(&mut self) {
        self.status = TransitionStatus::InActive;
    }

    pub fn state_snapshot(&self) -> CameraAnimationStateSnapshot {
        CameraAnimationStateSnapshot {
            active: self.is_active(),
            progress: self.progress(),
            duration_seconds: self.duration_seconds,
            elapsed_seconds: self.elapsed_seconds,
            target_coords: self.target_coords,
            easing: self.easing,
        }
    }

    fn complete(&mut self) {
        self.elapsed_seconds = self.duration_seconds;
        self.status = TransitionStatus::InActive;
    }

    fn current_position(&self) -> Coordinates3 {
        let eased_progress = self.easing.apply(self.progress());
        let start_position = self.start_coords.to_vec();
        let target_position = self.target_coords.to_vec();

        Coordinates3::from_vec3(start_position.lerp(target_position, eased_progress))
    }

    fn is_at_target(&self) -> bool {
        self.start_coords
            .to_vec()
            .distance(self.target_coords.to_vec())
            <= Self::COMPLETE_DISTANCE
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionStatus {
    Active,
    InActive,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_uses_explicit_duration_when_provided() {
        let request = CameraAnimationRequest::new(
            Coordinates3::new(0.0, 0.0, 10.0),
            Some(2.5),
            CameraAnimationEasing::EaseOut,
        );

        let actual = request.resolve_duration_seconds(Coordinates3::new(0.0, 0.0, 0.0), 20.0);

        assert_eq!(actual, 2.5);
    }

    #[test]
    fn test_request_derives_duration_from_camera_speed_when_missing() {
        let request = CameraAnimationRequest::from_target(Coordinates3::new(0.0, 0.0, 10.0));

        let actual = request.resolve_duration_seconds(Coordinates3::new(0.0, 0.0, 0.0), 20.0);

        assert!((actual - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_transition_advances_to_target_and_completes() {
        let request = CameraAnimationRequest::new(
            Coordinates3::new(0.0, 0.0, 10.0),
            Some(1.0),
            CameraAnimationEasing::Linear,
        );
        let mut transition = CameraTransition::new(Coordinates3::new(0.0, 0.0, 0.0), request, 20.0);

        let halfway = transition.advance(0.5);
        assert!(transition.is_active());
        assert!((halfway.z - 5.0).abs() < 0.0001);

        let target = transition.advance(0.5);
        assert!(!transition.is_active());
        assert!((transition.progress() - 1.0).abs() < 0.0001);
        assert!((target.z - 10.0).abs() < 0.0001);
    }

    #[test]
    fn test_transition_stop_marks_animation_inactive() {
        let request = CameraAnimationRequest::from_target(Coordinates3::new(0.0, 0.0, 10.0));
        let mut transition = CameraTransition::new(Coordinates3::new(0.0, 0.0, 0.0), request, 20.0);

        transition.advance(0.25);
        transition.stop();

        assert!(!transition.is_active());
        assert!(transition.progress() > 0.0);
    }

    #[test]
    fn test_inactive_snapshot_uses_current_position() {
        let snapshot = CameraAnimationStateSnapshot::inactive(Coordinates3::new(1.0, 2.0, 3.0));

        assert!(!snapshot.active);
        assert_eq!(snapshot.progress, 0.0);
        assert_eq!(snapshot.target_coords.x, 1.0);
        assert_eq!(snapshot.target_coords.y, 2.0);
        assert_eq!(snapshot.target_coords.z, 3.0);
        assert_eq!(snapshot.easing, CameraAnimationEasing::Linear);
    }
}
