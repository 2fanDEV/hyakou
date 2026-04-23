use glam::Vec3;

use crate::renderer::actions::{Action, CameraActions};

use hyakou_core::{
    animations::trajectory::calculate_direction_vector,
    components::camera::{
        camera::Camera,
        data_structures::{CameraAxes, CameraMode, CameraTransition},
    },
    types::{DeltaTime, mouse_delta::MouseDelta},
};

#[derive(Debug)]
pub struct CameraMovementHandler {
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_speed_modifier_pressed: bool,
    is_slow_modifier_pressed: bool,
    is_mouse_dragging: bool,
}

impl CameraMovementHandler {
    pub fn new() -> Self {
        Self {
            is_backward_pressed: false,
            is_forward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            is_speed_modifier_pressed: false,
            is_slow_modifier_pressed: false,
            is_mouse_dragging: false,
        }
    }

    pub fn mouse_movement(
        &mut self,
        camera: &mut Camera,
        mode: &CameraMode,
        mouse_delta: &MouseDelta,
        _delta_time: DeltaTime,
    ) {
        if self.is_mouse_dragging {
            match mode {
                CameraMode::PAN => {
                    let delta_x = mouse_delta.delta_position.x() as f32;
                    let delta_y = mouse_delta.delta_position.y() as f32;
                    let axes = self.get_axes(camera, mode);
                    let offset =
                        Self::calculate_pan_offset(delta_x, delta_y, &axes, camera.sensitivity);
                    camera.eye += offset;
                    camera.target += offset;
                }
                CameraMode::ORBIT => {
                    let yaw_delta = mouse_delta.delta_position.x() as f32;
                    let pitch_delta = mouse_delta.delta_position.y() as f32;
                    Self::rotate_orbit_camera(camera, yaw_delta, pitch_delta);
                }
                CameraMode::FLY => {
                    let yaw_delta = mouse_delta.delta_position.x() as f32;
                    let pitch_delta = mouse_delta.delta_position.y() as f32;
                    camera.move_camera(yaw_delta, pitch_delta);
                }
            }
        }
    }

    pub fn handle_action(&mut self, action: &Action, is_pressed: bool) {
        match action {
            Action::Camera(camera_action) => match camera_action {
                CameraActions::Forwards => self.is_forward_pressed = is_pressed,
                CameraActions::Backwards => self.is_backward_pressed = is_pressed,
                CameraActions::Left => self.is_left_pressed = is_pressed,
                CameraActions::Right => self.is_right_pressed = is_pressed,
                CameraActions::Up => self.is_up_pressed = is_pressed,
                CameraActions::Down => self.is_down_pressed = is_pressed,
                CameraActions::SpeedModifier => self.is_speed_modifier_pressed = is_pressed,
                CameraActions::SlowModifier => self.is_slow_modifier_pressed = is_pressed,
                CameraActions::Drag => self.is_mouse_dragging = is_pressed,
            },
        }
    }

    pub fn transition_camera_incrementally(
        &self,
        camera: &mut Camera,
        transition: &mut CameraTransition,
        delta_time: DeltaTime,
    ) {
        camera.eye = transition.advance(delta_time).to_vec();
    }

    pub fn update_camera_with_keyboard(
        &self,
        camera: &mut Camera,
        mode: &CameraMode,
        delta_time: DeltaTime,
    ) {
        let axes = self.get_axes(camera, mode);
        let speed = self.adjust_speed(camera.speed * delta_time);
        let movement = self.movement_calculcation(camera, mode, axes, speed);
        self.update_camera_with_movement(camera, mode, &movement);
    }

    fn update_camera_with_movement(&self, camera: &mut Camera, mode: &CameraMode, movement: &Vec3) {
        match mode {
            CameraMode::ORBIT => camera.eye += movement,
            _ => {
                camera.eye += movement;
                camera.target += movement;
            }
        }
    }

    fn get_axes(&self, camera: &Camera, mode: &CameraMode) -> CameraAxes {
        let (forward, forward_mag) = match mode {
            CameraMode::ORBIT => {
                let forward = camera.target - camera.eye;
                (forward.normalize(), forward.length())
            }
            CameraMode::FLY => (calculate_direction_vector(*camera.yaw, *camera.pitch), 1.0),
            CameraMode::PAN => {
                let forward = camera.target - camera.eye;
                (forward.normalize(), forward.length())
            }
        };

        let right = forward.cross(camera.up).normalize();
        let view_up = right.cross(forward).normalize();
        CameraAxes {
            forward,
            forward_mag,
            right,
            view_up,
        }
    }

    fn calculate_pan_offset(
        delta_x: f32,
        delta_y: f32,
        axes: &CameraAxes,
        sensitivity: f32,
    ) -> Vec3 {
        axes.view_up * (delta_y * sensitivity) - axes.right * (delta_x * sensitivity)
    }

    fn rotate_orbit_camera(camera: &mut Camera, yaw_delta: f32, pitch_delta: f32) {
        let orbit_radius = camera.eye.distance(camera.target);

        camera.yaw.add(
            yaw_delta * camera.sensitivity,
            camera.precalculated_smoothing,
            camera.smoothing_factor,
        );
        camera.pitch.add(
            pitch_delta * camera.sensitivity,
            camera.precalculated_smoothing,
            camera.smoothing_factor,
        );

        let forward = calculate_direction_vector(*camera.yaw, *camera.pitch);
        camera.eye = camera.target - forward * orbit_radius;
    }

    fn movement_calculcation(
        &self,
        camera: &Camera,
        mode: &CameraMode,
        axes: CameraAxes,
        speed: f32,
    ) -> Vec3 {
        let mut movement = Vec3::ZERO;
        self.calculate_forwards_movement(
            mode,
            &axes.forward,
            axes.forward_mag,
            &axes.view_up,
            speed,
            &mut movement,
        );
        self.calculate_backwards_movement(mode, &axes.forward, &axes.view_up, speed, &mut movement);
        self.calculate_right_movement(mode, &axes.right, speed, &mut movement);
        self.calculate_left_movement(mode, &axes.right, speed, &mut movement);
        self.calculate_up_movement(&camera.up, speed, &mut movement);
        self.calculate_down_movement(&camera.up, speed, &mut movement);
        movement
    }

    fn calculate_up_movement(&self, up: &Vec3, speed: f32, movement: &mut Vec3) {
        if self.is_up_pressed {
            *movement += up * speed;
        }
    }

    fn calculate_down_movement(&self, up: &Vec3, speed: f32, movement: &mut Vec3) {
        if self.is_down_pressed {
            *movement -= up * speed;
        }
    }

    fn calculate_forwards_movement(
        &self,
        mode: &CameraMode,
        forward: &Vec3,
        forward_mag: f32,
        view_up: &Vec3,
        speed: f32,
        movement: &mut Vec3,
    ) {
        if self.is_forward_pressed {
            match mode {
                CameraMode::FLY => *movement += forward * speed,
                CameraMode::ORBIT => {
                    if forward_mag > speed {
                        *movement += forward * speed;
                    }
                }
                CameraMode::PAN => *movement += view_up * speed,
            }
        }
    }

    fn calculate_left_movement(
        &self,
        mode: &CameraMode,
        right: &Vec3,
        speed_multiplier: f32,
        movement: &mut Vec3,
    ) {
        if self.is_left_pressed {
            match mode {
                CameraMode::PAN => *movement -= right * speed_multiplier,
                CameraMode::FLY => *movement -= right * speed_multiplier,
                CameraMode::ORBIT => *movement -= right * speed_multiplier,
            }
        }
    }
    fn calculate_right_movement(
        &self,
        mode: &CameraMode,
        right: &Vec3,
        speed_multiplier: f32,
        movement: &mut Vec3,
    ) {
        if self.is_right_pressed {
            match mode {
                CameraMode::PAN => *movement += right * speed_multiplier,
                CameraMode::FLY => *movement += right * speed_multiplier,
                CameraMode::ORBIT => *movement += right * speed_multiplier,
            }
        }
    }

    fn calculate_backwards_movement(
        &self,
        mode: &CameraMode,
        forward: &Vec3,
        view_up: &Vec3,
        speed_multiplier: f32,
        movement: &mut Vec3,
    ) {
        if self.is_backward_pressed {
            match mode {
                CameraMode::PAN => *movement -= view_up * speed_multiplier,
                CameraMode::FLY => *movement -= forward * speed_multiplier,
                CameraMode::ORBIT => *movement -= forward * speed_multiplier,
            }
        }
    }

    fn adjust_speed(&self, mut speed: f32) -> f32 {
        if self.is_slow_modifier_pressed {
            speed *= 0.5;
        }
        if self.is_speed_modifier_pressed {
            speed *= 2.0;
        }
        speed
    }
}

#[cfg(test)]
#[path = "movement_tests.rs"]
mod tests;
