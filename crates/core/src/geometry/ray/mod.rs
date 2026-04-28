use anyhow::{Result, anyhow};
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

pub fn ndc_to_world(camera: &Camera, ndc: Vec2, depth: f32) -> Option<Vec3> {
    if !ndc.is_finite() || !depth.is_finite() || !(0.0..=1.0).contains(&depth) {
        return None;
    }

    let inverse_view_projection = camera.build_view_proj_matrix().inverse();
    let world = inverse_view_projection * Vec4::new(ndc.x, ndc.y, depth, 1.0);

    if !world.w.is_finite() || world.w.abs() <= f32::EPSILON {
        return None;
    }

    let world = world.xyz() / world.w;
    world.is_finite().then_some(world)
}

pub fn ray_from_screen(camera: &Camera, x: f32, y: f32, size: Size) -> Result<Ray> {
    let ndc = match screen_to_ndc(x, y, size) {
        Some(ndc) => ndc,
        None => {
            return Err(anyhow!("Invalid screen coordinates, size={:?}", size));
        }
    };
    let world_near = ndc_to_world(camera, ndc, 0.0)
        .ok_or_else(|| anyhow!("Failed to unproject NDC coordinates: {ndc:?}"))?;

    Ok(Ray(camera.eye, (world_near - camera.eye).normalize()))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
