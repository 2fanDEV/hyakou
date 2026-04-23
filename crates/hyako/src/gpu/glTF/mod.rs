use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use hyakou_core::geometry::node::NodeGraph;

mod builder;
mod resources;

#[cfg(test)]
pub(super) use builder::{PrimitiveContext, ensure_indices_in_range};

#[derive(Debug, Clone)]
pub struct GLTFLoader;

#[derive(Debug, Clone)]
pub(super) struct ImportContext {
    pub(super) asset_label: String,
    pub(super) buffer_base_dir: Option<PathBuf>,
}

impl GLTFLoader {
    pub fn new() -> Self {
        Self
    }

    pub async fn load_from_path(&self, path: &Path) -> Result<NodeGraph> {
        let slice = resources::read_asset(path).await?;
        let context = ImportContext {
            asset_label: path.display().to_string(),
            buffer_base_dir: path.parent().map(Path::to_path_buf),
        };
        self.load_from_bytes_with_context(slice, context).await
    }

    pub async fn load_from_bytes(&self, slice: Vec<u8>) -> Result<NodeGraph> {
        self.load_from_bytes_with_label(slice, "in-memory glTF asset")
            .await
    }

    pub async fn load_from_bytes_with_label(
        &self,
        slice: Vec<u8>,
        asset_label: impl Into<String>,
    ) -> Result<NodeGraph> {
        let context = ImportContext {
            asset_label: asset_label.into(),
            buffer_base_dir: None,
        };
        self.load_from_bytes_with_context(slice, context).await
    }

    async fn load_from_bytes_with_context(
        &self,
        slice: Vec<u8>,
        context: ImportContext,
    ) -> Result<NodeGraph> {
        let gltf = gltf::Gltf::from_slice(&slice).map_err(|error| {
            anyhow!(
                "Failed to parse glTF asset `{}`: {error}",
                context.asset_label
            )
        })?;

        let buffer_data = resources::load_buffers(&gltf, &context).await?;
        builder::build_node_graph(&gltf, &buffer_data, &context.asset_label)
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
