use anyhow::{Result, anyhow};

use crate::renderer::{
    components::transform::Transform,
    types::{DeltaTime, ids::MeshId},
};

pub mod trajectory;

pub const NEUTRAL_SPEED: f32 = 1.0;

/// A trait to implement when specific trajectory path are to be implemented.
/// The animate(...) most likely uses a try_wtite on a Arc<RwLock<Transform>> which could
/// panic but should be handled gracefully. Nonetheless you can match the result to get the
/// error that occurs when try_write fails to acquire the lock.
pub trait Animation: Send {
    fn get_id(&self) -> &MeshId;
    fn animate(&mut self, t: Option<&Transform>, delta: DeltaTime) -> Result<()>;
    fn reset(&mut self);
}

pub struct Animator {
    pub id: MeshId,
    elapsed_time: f32,
    speed_multiplier: f32,
    pub is_currently_playing: bool,
    animation: Box<dyn Animation>,
}

impl Animator {
    pub fn new(speed_multiplier: f32, animation: Box<dyn Animation>) -> Result<Self> {
        Ok(Self {
            id: animation.get_id().to_owned(),
            speed_multiplier,
            elapsed_time: 0.0,
            is_currently_playing: true,
            animation,
        })
    }

    pub fn play(&mut self, delta_time: DeltaTime) -> Result<()> {
        if self.is_currently_playing {
            self.elapsed_time += delta_time;
            if let Err(e) = self
                .animation
                .animate(None, self.speed_multiplier * delta_time)
            {
                return Err(anyhow!(
                    "Error at animator {:?} with the following message: {:?}",
                    self.id,
                    e
                ));
            }
        }
        Ok(())
    }

    pub fn resume(&mut self) {
        self.is_currently_playing = true;
    }

    pub fn pause(&mut self) {
        self.is_currently_playing = false;
    }

    pub fn reset(&mut self) {
        self.elapsed_time = 0.0;
        self.animation.reset();
    }

    pub fn get_elapsed_time(&self) -> f32 {
        self.elapsed_time
    }

    pub fn get_speed_multiplier(&self) -> f32 {
        self.speed_multiplier
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // Mock animation for testing that tracks calls and delta times
    struct MockAnimation {
        id: MeshId,
        animate_calls: Arc<Mutex<Vec<f32>>>,
        reset_calls: Arc<Mutex<usize>>,
    }

    impl MockAnimation {
        fn new() -> (Self, Arc<Mutex<Vec<f32>>>, Arc<Mutex<usize>>) {
            let animate_calls = Arc::new(Mutex::new(Vec::new()));
            let reset_calls = Arc::new(Mutex::new(0));

            let mock = Self {
                id: MeshId("test_animator".to_string()),
                animate_calls: animate_calls.clone(),
                reset_calls: reset_calls.clone(),
            };

            (mock, animate_calls, reset_calls)
        }
    }

    impl Animation for MockAnimation {
        fn get_id(&self) -> &MeshId {
            &self.id
        }

        fn animate(&mut self, _t: Option<&Transform>, delta: DeltaTime) -> Result<()> {
            self.animate_calls.lock().unwrap().push(delta);
            Ok(())
        }

        fn reset(&mut self) {
            *self.reset_calls.lock().unwrap() += 1;
        }
    }

    #[test]
    fn test_time_accumulation_single_frame() {
        let (mock, _, _) = MockAnimation::new();
        let mut animator = Animator::new(NEUTRAL_SPEED, Box::new(mock)).unwrap();

        animator.play(0.016).unwrap();

        assert_eq!(animator.get_elapsed_time(), 0.016);
    }

    #[test]
    fn test_time_accumulation_multiple_frames() {
        let (mock, _, _) = MockAnimation::new();
        let mut animator = Animator::new(NEUTRAL_SPEED, Box::new(mock)).unwrap();

        animator.play(0.016).unwrap();
        animator.play(0.016).unwrap();
        animator.play(0.020).unwrap();

        assert!((animator.get_elapsed_time() - 0.052).abs() < 0.0001);
    }

    #[test]
    fn test_time_does_not_accumulates_when_paused() {
        let (mock, _, _) = MockAnimation::new();
        let mut animator = Animator::new(NEUTRAL_SPEED, Box::new(mock)).unwrap();

        animator.play(0.016).unwrap();
        animator.pause();
        animator.play(0.016).unwrap();

        // Time should still accumulate even when paused
        assert!((animator.get_elapsed_time() - 0.032).abs() < 0.017);
    }

    #[test]
    fn test_pause_stops_animation() {
        let (mock, animate_calls, _) = MockAnimation::new();
        let mut animator = Animator::new(NEUTRAL_SPEED, Box::new(mock)).unwrap();

        animator.play(0.016).unwrap();
        assert_eq!(animate_calls.lock().unwrap().len(), 1);

        animator.pause();
        animator.play(0.016).unwrap();

        // Should still only have 1 call because it was paused
        assert_eq!(animate_calls.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_resume_continues_animation() {
        let (mock, animate_calls, _) = MockAnimation::new();
        let mut animator = Animator::new(NEUTRAL_SPEED, Box::new(mock)).unwrap();

        animator.play(0.016).unwrap();
        animator.pause();
        animator.play(0.016).unwrap();
        assert_eq!(animate_calls.lock().unwrap().len(), 1);

        animator.resume();
        animator.play(0.016).unwrap();

        assert_eq!(animate_calls.lock().unwrap().len(), 2);
    }

    #[test]
    fn test_starts_playing_by_default() {
        let (mock, animate_calls, _) = MockAnimation::new();
        let mut animator = Animator::new(NEUTRAL_SPEED, Box::new(mock)).unwrap();

        assert_eq!(animator.is_currently_playing, true);
        animator.play(0.016).unwrap();
        assert_eq!(animate_calls.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_speed_multiplier_neutral() {
        let (mock, animate_calls, _) = MockAnimation::new();
        let mut animator = Animator::new(NEUTRAL_SPEED, Box::new(mock)).unwrap();

        animator.play(0.016).unwrap();

        let calls = animate_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert!((calls[0] - 0.016).abs() < 0.0001);
    }

    #[test]
    fn test_speed_multiplier_double() {
        let (mock, animate_calls, _) = MockAnimation::new();
        let mut animator = Animator::new(2.0, Box::new(mock)).unwrap();

        animator.play(0.016).unwrap();

        let calls = animate_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert!((calls[0] - 0.032).abs() < 0.0001); // 0.016 * 2.0
    }

    #[test]
    fn test_speed_multiplier_half() {
        let (mock, animate_calls, _) = MockAnimation::new();
        let mut animator = Animator::new(0.5, Box::new(mock)).unwrap();

        animator.play(0.016).unwrap();

        let calls = animate_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert!((calls[0] - 0.008).abs() < 0.0001); // 0.016 * 0.5
    }

    #[test]
    fn test_speed_multiplier_preserved_across_calls() {
        let (mock, animate_calls, _) = MockAnimation::new();
        let mut animator = Animator::new(3.0, Box::new(mock)).unwrap();

        animator.play(0.016).unwrap();
        animator.play(0.020).unwrap();

        let calls = animate_calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert!((calls[0] - 0.048).abs() < 0.0001); // 0.016 * 3.0
        assert!((calls[1] - 0.060).abs() < 0.0001); // 0.020 * 3.0
    }

    #[test]
    fn test_reset_clears_elapsed_time() {
        let (mock, _, _) = MockAnimation::new();
        let mut animator = Animator::new(NEUTRAL_SPEED, Box::new(mock)).unwrap();

        animator.play(0.016).unwrap();
        animator.play(0.016).unwrap();
        assert!((animator.get_elapsed_time() - 0.032).abs() < 0.0001);

        animator.reset();

        assert_eq!(animator.get_elapsed_time(), 0.0);
    }

    #[test]
    fn test_reset_calls_animation_reset() {
        let (mock, _, reset_calls) = MockAnimation::new();
        let mut animator = Animator::new(NEUTRAL_SPEED, Box::new(mock)).unwrap();

        animator.reset();

        assert_eq!(*reset_calls.lock().unwrap(), 1);
    }

    #[test]
    fn test_reset_multiple_times() {
        let (mock, _, reset_calls) = MockAnimation::new();
        let mut animator = Animator::new(NEUTRAL_SPEED, Box::new(mock)).unwrap();

        animator.play(0.016).unwrap();
        animator.reset();
        animator.play(0.020).unwrap();
        animator.reset();
        animator.reset();

        assert_eq!(*reset_calls.lock().unwrap(), 3);
        assert_eq!(animator.get_elapsed_time(), 0.0);
    }

    #[test]
    fn test_reset_preserves_speed_multiplier() {
        let (mock, _, _) = MockAnimation::new();
        let mut animator = Animator::new(2.5, Box::new(mock)).unwrap();

        animator.reset();

        assert_eq!(animator.get_speed_multiplier(), 2.5);
    }

    #[test]
    fn test_reset_preserves_playing_state() {
        let (mock, _, _) = MockAnimation::new();
        let mut animator = Animator::new(NEUTRAL_SPEED, Box::new(mock)).unwrap();

        animator.pause();
        animator.reset();

        assert_eq!(animator.is_currently_playing, false);
    }
}
