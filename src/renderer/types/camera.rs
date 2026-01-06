use std::{
    f32::consts::{PI, TAU},
    ops::Deref,
};

use crate::renderer::types::F32_ZERO;

fn smoothing_interpolation(
    prev_value: f32,
    delta: f32,
    precalculated_smoothing_factor: f32,
    smoothing_factor: f32,
) -> f32 {
    prev_value * precalculated_smoothing_factor + delta * smoothing_factor
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Yaw {
    value: f32,
    previous_delta: f32,
}

impl Yaw {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            previous_delta: F32_ZERO,
        }
    }

    pub fn add(&mut self, value: f32, one_minus_smoothing_value: f32, smoothing_factor: f32) {
        let smoothed_delta_interpolation = smoothing_interpolation(
            self.previous_delta,
            value,
            one_minus_smoothing_value,
            smoothing_factor,
        );

        if self.value > PI {
            self.value -= TAU;
        } else {
            self.value += smoothed_delta_interpolation;
        }
        self.previous_delta = smoothed_delta_interpolation;
    }
}

impl Deref for Yaw {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Pitch {
    value: f32,
    previous_delta: f32,
}

impl Pitch {
    const PITCH_CLAMP: f32 = 89.0_f32;

    pub fn new(value: f32) -> Self {
        Self {
            value,
            previous_delta: F32_ZERO,
        }
    }

    pub fn add(&mut self, value: f32, one_minus_smoothing_value: f32, smoothing_factor: f32) {
        let smoothed_interpolation_value = smoothing_interpolation(
            self.previous_delta,
            value,
            one_minus_smoothing_value,
            smoothing_factor,
        );
        self.value = (self.value - smoothed_interpolation_value).clamp(
            -Self::PITCH_CLAMP.to_radians(),
            Self::PITCH_CLAMP.to_radians(),
        );

        self.previous_delta = smoothed_interpolation_value;
    }
}

impl Deref for Pitch {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaw_add_positive_delta() {
        let mut yaw = Yaw::new(0.0);
        let initial_value = *yaw;

        // Add positive delta with smoothing factor 0.5
        // one_minus_smoothing = 1.0 - 0.5 = 0.5
        yaw.add(10.0, 0.5, 0.5);

        // First add: smoothed_delta = 0.0 * 0.5 + 10.0 * 0.5 = 5.0
        // value = 0.0 + 5.0 = 5.0
        assert!(
            *yaw > initial_value,
            "Yaw should increase with positive delta. Initial: {}, New: {}",
            initial_value,
            *yaw
        );
        assert_eq!(*yaw, 5.0, "Yaw should be 5.0 after first smoothed add");
    }

    #[test]
    fn test_yaw_add_negative_delta() {
        let mut yaw = Yaw::new(0.0);
        let initial_value = *yaw;

        // Add negative delta with smoothing factor 0.5
        yaw.add(-10.0, 0.5, 0.5);

        // First add: smoothed_delta = 0.0 * 0.5 + (-10.0) * 0.5 = -5.0
        // value = 0.0 + (-5.0) = -5.0
        assert!(
            *yaw < initial_value,
            "Yaw should decrease with negative delta. Initial: {}, New: {}",
            initial_value,
            *yaw
        );
        assert_eq!(*yaw, -5.0, "Yaw should be -5.0 after first smoothed add");
    }

    #[test]
    fn test_pitch_add_positive_delta() {
        let mut pitch = Pitch::new(0.0);
        let initial_value = *pitch;

        // Add positive delta with smoothing factor 0.5
        // Pitch uses subtraction (inverted Y-axis)
        pitch.add(2.0, 0.5, 0.5);

        // First add: smoothed_delta = 0.0 * 0.5 + 2.0 * 0.5 = 1.0
        // value = 0.0 - 1.0 = -1.0 (subtraction for inverted Y-axis)
        assert!(
            *pitch < initial_value,
            "Pitch should decrease with positive delta (inverted Y). Initial: {}, New: {}",
            initial_value,
            *pitch
        );
        assert_eq!(
            *pitch, -1.0,
            "Pitch should be -1.0 after first smoothed add"
        );
    }

    #[test]
    fn test_pitch_add_negative_delta() {
        let mut pitch = Pitch::new(0.0);
        let initial_value = *pitch;

        // Add negative delta with smoothing factor 0.5
        // Pitch uses subtraction (inverted Y-axis)
        pitch.add(-2.0, 0.5, 0.5);

        // First add: smoothed_delta = 0.0 * 0.5 + (-2.0) * 0.5 = -1.0
        // value = 0.0 - (-1.0) = 1.0 (subtracting negative = addition)
        assert!(
            *pitch > initial_value,
            "Pitch should increase with negative delta (inverted Y). Initial: {}, New: {}",
            initial_value,
            *pitch
        );
        assert_eq!(*pitch, 1.0, "Pitch should be 1.0 after first smoothed add");
    }
}
