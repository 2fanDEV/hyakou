use std::f32::consts::PI;
use std::sync::RwLock;

use glam::Vec3;
use hyakou_core::{
    components::camera::{
        camera::Camera,
        data_structures::{CameraAnimationRequest, CameraAnimationStateSnapshot, CameraMode},
    },
    types::{DeltaTime64, Size, camera::Pitch, mouse_delta::MouseDelta},
};

use crate::renderer::{
    handlers::InputEvent,
    handlers::camera::CameraHandler,
};

pub struct CameraController {
    inner: RwLock<CameraControllerInner>,
}

struct CameraControllerInner {
    camera: Camera,
    handler: CameraHandler,
}

impl CameraController {
    pub fn new(viewport_size: Size) -> Self {
        const CAMERA_SPEED_UNITS_PER_SECOND: f32 = 20.0;
        const CAMERA_SENSITIVITY: f32 = 0.001;
        let aspect = Camera::aspect_ratio_from_size(viewport_size);
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 15.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::Y,
            aspect,
            45.0_f32.to_radians(),
            0.1,
            1000.0,
            hyakou_core::types::camera::Yaw::new(-PI / 2.0),
            Pitch::new(0.0),
            CAMERA_SPEED_UNITS_PER_SECOND,
            CAMERA_SENSITIVITY,
            0.5,
        );

        Self {
            inner: RwLock::new(CameraControllerInner {
                camera,
                handler: CameraHandler::new(CameraMode::ORBIT),
            }),
        }
    }

    fn read_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&CameraControllerInner) -> R,
    {
        f(&self.inner.read().expect("CameraController lock poisoned"))
    }

    fn write_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut CameraControllerInner) -> R,
    {
        f(&mut self.inner.write().expect("CameraController lock poisoned"))
    }

    pub fn active_camera(&self) -> Camera {
        self.read_inner(|inner| inner.camera.clone())
    }

    pub fn camera_animation_state(&self) -> CameraAnimationStateSnapshot {
        self.read_inner(|inner| {
            inner
                .handler
                .state
                .camera_animation_state(&inner.camera)
        })
    }

    pub fn animate_camera(&self, request: CameraAnimationRequest) {
        self.write_inner(|inner| {
            inner
                .handler
                .state
                .animate_camera(&inner.camera, request);
        });
    }

    pub fn stop_camera_animation(&self) {
        self.write_inner(|inner| {
            inner
                .handler
                .state
                .stop_camera_animation(&inner.camera.id);
        });
    }

    pub fn set_camera_mode(&self, mode: CameraMode) {
        self.write_inner(|inner| inner.handler.set_mode(mode));
    }

    pub fn handle_input_events(&self, events: impl IntoIterator<Item = InputEvent>) {
        self.write_inner(|inner| {
            for event in events {
                let (action, is_pressed) = match event {
                    InputEvent::ActionStarted(action) => (action, true),
                    InputEvent::ActionEnded(action) => (action, false),
                };
                inner.handler.handle_action(&action, is_pressed);
            }
        });
    }

    pub fn handle_mouse_movement(&self, mouse_delta: &MouseDelta, dt: f32) {
        self.write_inner(|inner| {
            inner
                .handler
                .mouse_movement(&mut inner.camera, mouse_delta, dt);
        });
    }

    pub fn update(&self, dt: DeltaTime64) {
        self.write_inner(|inner| {
            inner.handler.update(&mut inner.camera, dt as f32);
        });
    }

    pub fn set_aspect_from_size(&self, size: Size) {
        if !size.is_zero() {
            self.write_inner(|inner| inner.camera.set_aspect_from_size(size));
        }
    }
}
