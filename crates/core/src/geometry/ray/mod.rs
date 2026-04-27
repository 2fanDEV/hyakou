use anyhow::Result;
use glam::{Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::{components::camera::camera::Camera, types::Size};

pub struct Ray(Vec3, Vec3);

impl Ray {
    pub fn origin(&self) -> Vec3 {
        self.0
    }

    pub fn direction(&self) -> Vec3 {
        self.1
    }
}

pub fn screen_to_ndc(x: f32, y: f32, size: Size) -> Option<Vec2> {
    if size.width == 0 || size.height == 0 {
        return None;
    }
    Some(Vec2::new(
        (2.0 * x / size.width as f32) - 1.0,
        1.0 - (2.0 * y / size.height as f32),
    ))
}

pub fn ray_from_screen(camera: &Camera, x: f32, y: f32, size: Size) -> Result<Ray> {
    let world_proj = camera.build_view_proj_matrix();
    let ndc = match screen_to_ndc(x, y, size) {
        Some(ndc) => ndc,
        None => {
            return Err(anyhow::anyhow!(
                "Invalid screen coordinates, size={:?}",
                size
            ));
        }
    };
    let inverse_world = world_proj.inverse();
    let near_ndc = Vec4::new(ndc.x, ndc.y, 0.0, 1.0);
    let near_world = inverse_world * near_ndc;

    let world_near = near_world.xyz() / near_world.w;

    Ok(Ray(camera.eye, (world_near - camera.eye).normalize()))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
