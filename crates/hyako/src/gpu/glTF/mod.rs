use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};

mod builder;
mod materials;
mod resources;
mod types;

#[cfg(test)]
pub(super) use builder::{PrimitiveContext, ensure_indices_in_range};
pub use types::{
    ImportedAlphaMode, ImportedImage, ImportedMagFilter, ImportedMaterial, ImportedMinFilter,
    ImportedSampler, ImportedScene, ImportedTexture, ImportedTextureRef, ImportedWrapMode,
};

#[derive(Debug, Clone)]
pub struct GLTFLoader;

#[derive(Debug, Clone)]
pub(super) struct ImportContext {
    pub(super) asset_label: String,
    pub(super) buffer_base_dir: Option<PathBuf>,
    pub(super) bundled_files: Option<HashMap<String, Vec<u8>>>,
}

impl GLTFLoader {
    pub fn new() -> Self {
        Self
    }

    pub async fn load_from_path(&self, path: &Path) -> Result<ImportedScene> {
        let slice = resources::read_asset(path).await?;
        let context = ImportContext {
            asset_label: path.display().to_string(),
            buffer_base_dir: path.parent().map(Path::to_path_buf),
            bundled_files: None,
        };
        self.load_from_bytes_with_context(slice, context).await
    }

    pub async fn load_from_bytes(&self, slice: Vec<u8>) -> Result<ImportedScene> {
        self.load_from_bytes_with_label(slice, "in-memory glTF asset")
            .await
    }

    pub async fn load_from_bytes_with_label(
        &self,
        slice: Vec<u8>,
        asset_label: impl Into<String>,
    ) -> Result<ImportedScene> {
        let context = ImportContext {
            asset_label: asset_label.into(),
            buffer_base_dir: None,
            bundled_files: None,
        };
        self.load_from_bytes_with_context(slice, context).await
    }

    pub async fn load_from_file_bundle(
        &self,
        entry_file_name: &str,
        files: Vec<(String, Vec<u8>)>,
    ) -> Result<ImportedScene> {
        let bundled_files = resources::build_uploaded_file_map(files)?;
        let entry_file = resources::resolve_uploaded_file(entry_file_name, &bundled_files)
            .ok_or_else(|| anyhow!("Missing bundle entry file `{entry_file_name}`"))?
            .clone();
        let context = ImportContext {
            asset_label: entry_file_name.to_string(),
            buffer_base_dir: None,
            bundled_files: Some(bundled_files),
        };

        self.load_from_bytes_with_context(entry_file, context).await
    }

    async fn load_from_bytes_with_context(
        &self,
        slice: Vec<u8>,
        context: ImportContext,
    ) -> Result<ImportedScene> {
        let gltf = gltf::Gltf::from_slice(&slice).map_err(|error| {
            anyhow!(
                "Failed to parse glTF asset `{}`: {error}",
                context.asset_label
            )
        })?;

        let buffer_data = resources::load_buffers(&gltf, &context).await?;
        let images = resources::load_images(&gltf, &buffer_data, &context).await?;
        let textures = materials::load_textures(&gltf);
        let samplers = materials::load_samplers(&gltf);
        let materials = materials::load_materials(&gltf)?;
        let node_graph = builder::build_node_graph(&gltf, &buffer_data, &context.asset_label)?;

        Ok(ImportedScene::new(
            node_graph, materials, images, textures, samplers,
        ))
    }
}

impl Default for GLTFLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../gltf_tests.rs"]
mod tests;
