use std::{ops::Deref, rc::Rc};

use crate::renderer::types::{F32_ZERO, F64_ZERO};

fn smoothing_interpolation(
    prev_value: f32,
    delta: f32,
    precalculated_smoothing_factor: f32,
    smoothing_factor: f32,
) -> f32 {
    prev_value * (1.0 - smoothing_factor) + delta * smoothing_factor
}

#[derive(Debug, Default)]
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

    pub fn add(&mut self, value: f32, precalculated_smoothing_value: f32, smoothing_factor: f32) {
        let smoothed_delta_interpolation = smoothing_interpolation(
            self.previous_delta,
            value,
            precalculated_smoothing_value,
            smoothing_factor,
        );
        self.value += smoothed_delta_interpolation;
        self.previous_delta = smoothed_delta_interpolation;
    }
}

impl Deref for Yaw {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Debug, Default)]
pub struct Pitch {
    value: f32,
    previous_delta: f32,
}

impl Pitch {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            previous_delta: F32_ZERO,
        }
    }

    pub fn add(&mut self, value: f32, precalculated_smoothing_value: f32, smoothing_factor: f32) {
        let smoothed_interpolation_value = smoothing_interpolation(
            self.previous_delta,
            value,
            self.one_minus_smoothing_factor,
            *self.smoothing_factor,
        );
        self.value = (self.value - smoothed_interpolation_value)
            .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        self.previous_delta = smoothed_interpolation_value;
    }
}

impl Deref for Pitch {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
