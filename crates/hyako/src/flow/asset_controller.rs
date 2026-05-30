use std::rc::Rc;

use hyakou_core::{
    components::AssetType,
    geometry::ray::Ray,
    selection::structure::{SelectionScope, SelectionTarget},
    types::ids::MeshId,
};

use crate::{
    gpu::{glTF::ImportedScene, render_mesh::RenderMesh},
    renderer::{handlers::asset_handler::AssetHandler, renderer_context::RenderContext},
};

pub struct AssetController {
    handler: AssetHandler,
}

impl AssetController {
    fn new(handler: AssetHandler) -> Self {
        Self { handler }
    }

    pub(crate) fn from_render_context(ctx: &RenderContext) -> Self {
        Self::new(AssetHandler::from_render_context(ctx))
    }

    pub(crate) fn upload_imported_scene(
        &mut self,
        id: String,
        asset_type: AssetType,
        imported_scene: ImportedScene,
    ) -> Option<Rc<RenderMesh>> {
        self.handler
            .upload_imported_scene(id, asset_type, imported_scene)
    }

    pub(crate) fn resolve_selection_target(
        &self,
        ray: &Ray,
        scope: SelectionScope,
    ) -> Option<SelectionTarget> {
        self.handler.resolve_selection_target(ray, scope)
    }

    pub(crate) fn get_visible_asset(&self, id: &MeshId) -> Option<&Rc<RenderMesh>> {
        self.handler.get_visible_asset(id)
    }

    pub(crate) fn visible_meshes_with_asset_type(
        &mut self,
        asset_type: &AssetType,
    ) -> impl Iterator<Item = &Rc<RenderMesh>> {
        self.handler.visible_meshes_with_asset_type(asset_type)
    }
}
