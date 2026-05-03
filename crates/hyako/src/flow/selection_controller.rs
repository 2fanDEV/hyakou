use anyhow::Result;
use hyakou_core::{
    components::camera::camera::Camera,
    geometry::ray::{Ray, ray_from_screen},
    selection::structure::{SelectionScope, SelectionTarget},
    types::{Size, ids::MeshId},
};
use log::{debug, warn};

use crate::selection::manager::SelectionManager;

pub trait SelectionSurface {
    fn active_camera(&self) -> Result<Camera>;
    fn viewport_size(&self) -> Result<Size>;
    fn resolve_selection_target(&self, ray: Ray, scope: SelectionScope) -> Option<SelectionTarget>;
    fn set_outlined_meshes(&self, mesh_ids: Vec<MeshId>);
    fn clear_outlined_meshes(&self);
}

pub struct SelectionController {
    selection_manager: SelectionManager,
}

impl SelectionController {
    pub fn new() -> Self {
        Self {
            selection_manager: SelectionManager::new(),
        }
    }

    pub fn generate_ray(&self, x: f32, y: f32, size: &Size, camera: &Camera) -> Option<Ray> {
        match ray_from_screen(camera, x, y, size) {
            Ok(ray) => Some(ray),
            Err(e) => {
                debug!("{:?}", e);
                None
            }
        }
    }

    pub fn select_at_screen_point(
        &mut self,
        surface: &impl SelectionSurface,
        x: f32,
        y: f32,
        scope: SelectionScope,
    ) {
        let camera = match surface.active_camera() {
            Ok(camera) => camera,
            Err(error) => {
                warn!("Skipping selection because active camera is unavailable: {error:?}");
                return;
            }
        };

        let size = match surface.viewport_size() {
            Ok(size) => size,
            Err(error) => {
                warn!("Skipping selection because viewport size is unavailable: {error:?}");
                return;
            }
        };

        let Some(ray) = self.generate_ray(x, y, &size, &camera) else {
            return;
        };

        let Some(target) = surface.resolve_selection_target(ray, scope) else {
            self.clear();
            surface.clear_outlined_meshes();
            return;
        };

        let outline_mesh_ids = target.outline_mesh_ids().clone();
        self.select(target);
        surface.set_outlined_meshes(outline_mesh_ids);
    }

    pub fn select(&mut self, target: SelectionTarget) {
        self.selection_manager.select_single(target);
    }

    pub fn deselect(&mut self, target: SelectionTarget) {
        self.selection_manager.deselect(target);
    }

    pub fn is_selected(&self, mesh_id: &MeshId) -> bool {
        self.selection_manager
            .current_selected()
            .iter()
            .filter(|target| target.mesh_id().eq(mesh_id))
            .count()
            > 0
            || self
                .selection_manager
                .current_outline_selected()
                .iter()
                .filter(|mesh| mesh_id.eq(mesh))
                .count()
                > 0
    }

    pub fn clear(&mut self) {
        self.selection_manager.clear();
    }
}
