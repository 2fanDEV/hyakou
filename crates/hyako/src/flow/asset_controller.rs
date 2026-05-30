use std::{collections::hash_set::Iter, path::Path, rc::Rc};

use anyhow::Result;
use hyakou_core::{
    components::AssetType,
    geometry::ray::Ray,
    selection::structure::{SelectionScope, SelectionTarget},
    types::ids::MeshId,
};

use crate::{
    gpu::{glTF::ImportedScene, render_mesh::RenderMesh},
    renderer::handlers::asset_handler::AssetHandler,
};

pub struct AssetController {
    handler: AssetHandler,
}

impl AssetController {
    pub fn new(handler: AssetHandler) -> Self {
        Self { handler }
    }

    pub async fn upload_from_bytes(
        &mut self,
        id: String,
        asset_type: AssetType,
        bytes: Vec<u8>,
    ) -> Result<()> {
        self.handler.upload_from_bytes(id, asset_type, bytes).await
    }

    pub fn upload_imported_scene(
        &mut self,
        id: String,
        asset_type: AssetType,
        imported_scene: ImportedScene,
    ) -> Option<Rc<RenderMesh>> {
        self.handler
            .upload_imported_scene(id, asset_type, imported_scene)
    }

    pub async fn add_from_path(
        &mut self,
        id: String,
        light_type: AssetType,
        path: &Path,
    ) -> Result<Rc<RenderMesh>> {
        self.handler.add_from_path(id, light_type, path).await
    }

    pub fn resolve_selection_target(
        &self,
        ray: &Ray,
        scope: SelectionScope,
    ) -> Option<SelectionTarget> {
        self.handler.resolve_selection_target(ray, scope)
    }

    pub fn toggle_visibility(&mut self, id: String) {
        self.handler.toggle_visibility(id)
    }

    pub fn get_all_loaded_asset_ids(&self) -> Vec<String> {
        self.handler.get_all_loaded_asset_ids()
    }

    pub fn get_visible_asset_ids(&self) -> Iter<'_, String> {
        self.handler.get_visible_asset_ids()
    }

    pub fn get_visible_asset(&self, id: &MeshId) -> Option<&Rc<RenderMesh>> {
        self.handler.get_visible_asset(id)
    }

    pub fn get_all_visible_assets(&self) -> impl Iterator<Item = &Rc<RenderMesh>> {
        self.handler.get_all_visible_assets()
    }

    pub fn get_all_visible_assets_with_modifier(
        &mut self,
        light_type: &AssetType,
    ) -> impl Iterator<Item = &Rc<RenderMesh>> {
        self.handler
            .get_all_visible_assets_with_modifier(light_type)
    }
}
