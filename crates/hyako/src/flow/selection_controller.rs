use anyhow::Result;
use hyakou_core::{
    components::camera::camera::Camera,
    geometry::ray::{Ray, ray_from_screen},
    selection::structure::{SelectionScope, SelectionTarget},
    types::{Size, ids::MeshId},
};
use log::{debug, warn};

use crate::{flow::SceneFrameInput, selection::manager::SelectionManager};

pub trait SelectionContext {
    fn active_camera(&self) -> Result<Camera>;
    fn viewport_size(&self) -> Result<Size>;
    fn resolve_selection_target(&self, ray: Ray, scope: SelectionScope) -> Option<SelectionTarget>;
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
        context: &impl SelectionContext,
        x: f32,
        y: f32,
        scope: SelectionScope,
    ) {
        let camera = match context.active_camera() {
            Ok(camera) => camera,
            Err(error) => {
                warn!("Skipping selection because active camera is unavailable: {error:?}");
                return;
            }
        };

        let size = match context.viewport_size() {
            Ok(size) => size,
            Err(error) => {
                warn!("Skipping selection because viewport size is unavailable: {error:?}");
                return;
            }
        };

        let Some(ray) = self.generate_ray(x, y, &size, &camera) else {
            return;
        };

        let Some(target) = context.resolve_selection_target(ray, scope) else {
            self.clear();
            return;
        };

        self.select(target);
    }

    pub fn select(&mut self, target: SelectionTarget) {
        self.selection_manager.select(target);
    }

    pub fn clear(&mut self) {
        self.selection_manager.clear();
    }

    pub fn current_selection(&self) -> &[SelectionTarget] {
        self.selection_manager.current_selection()
    }

    pub fn outlined_mesh_ids(&self) -> &[MeshId] {
        self.selection_manager.outline_selection()
    }

    pub fn scene_frame_input(&self) -> SceneFrameInput<'_> {
        SceneFrameInput {
            outlined_mesh_ids: self.outlined_mesh_ids(),
        }
    }
}
