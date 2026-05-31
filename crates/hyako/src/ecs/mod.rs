use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};
use hyakou_core::{components::camera::camera::Camera, types::transform::Transform};
use std::sync::Arc;

use crate::{flow::CameraController, renderer::SceneRenderer};

#[derive(Resource)]
pub struct SceneRendererHandle(pub Arc<SceneRenderer>);

#[derive(Resource)]
pub struct CameraControllerHandle(pub Arc<CameraController>);

pub fn print_camera_position(query: Query<&Transform, With<Camera>>) {
    for transform in &query {
        log::info!("ECS camera position: {:?}", transform.position);
    }
}

pub fn init_world() -> (World, Schedule) {
    let mut world = World::new();

    world.spawn((
        Transform::new(Vec3::new(0.0, 0.0, 15.0), Quat::IDENTITY, Vec3::ONE),
        Camera::default(),
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems(print_camera_position);

    (world, schedule)
}
