use anyhow::Result;
use hyakou_core::{components::AssetType, types::import_diagnostic::ImportDiagnostic};
use log::{debug, error, warn};
#[cfg(target_arch = "wasm32")]
use shared::{Shared, SharedAccess};

use crate::{
    flow::{FlowCommand, FlowCommandSender},
    gpu::glTF::{GLTFLoader, ImportedScene},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

pub struct AssetUploadController {
    commands: FlowCommandSender,
    #[cfg(target_arch = "wasm32")]
    upload_status_callback: Shared<Option<js_sys::Function>>,
}

enum AssetUploadSource {
    Bytes(Vec<u8>),
    Bundle(Vec<(String, Vec<u8>)>),
}

impl AssetUploadController {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(commands: FlowCommandSender) -> Self {
        Self { commands }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(
        commands: FlowCommandSender,
        upload_status_callback: Shared<Option<js_sys::Function>>,
    ) -> Self {
        Self {
            commands,
            upload_status_callback,
        }
    }

    pub fn handle_asset_upload_requested(
        &self,
        id: String,
        file_name: String,
        asset_type: AssetType,
        bytes: Vec<u8>,
    ) {
        self.handle_upload_requested(id, file_name, asset_type, AssetUploadSource::Bytes(bytes));
    }

    pub fn handle_asset_bundle_upload_requested(
        &self,
        id: String,
        file_name: String,
        asset_type: AssetType,
        files: Vec<(String, Vec<u8>)>,
    ) {
        self.handle_upload_requested(id, file_name, asset_type, AssetUploadSource::Bundle(files));
    }

    fn handle_upload_requested(
        &self,
        id: String,
        file_name: String,
        asset_type: AssetType,
        source: AssetUploadSource,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let parsed_scene = pollster::block_on(Self::parse_uploaded_asset(&file_name, source));
            match parsed_scene {
                Ok(node_graph) => {
                    self.send_command(FlowCommand::ApplyParsedAsset {
                        id,
                        file_name,
                        asset_type,
                        imported_scene: node_graph,
                    });
                }
                Err(upload_error) => {
                    self.send_command(FlowCommand::AssetUploadFailed {
                        id,
                        file_name,
                        error: upload_error.to_string(),
                    });
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let commands = self.commands.clone();
            spawn_local(async move {
                let parsed_scene = Self::parse_uploaded_asset(&file_name, source).await;
                let next_command = match parsed_scene {
                    Ok(node_graph) => FlowCommand::ApplyParsedAsset {
                        id,
                        file_name,
                        asset_type,
                        imported_scene: node_graph,
                    },
                    Err(upload_error) => FlowCommand::AssetUploadFailed {
                        id,
                        file_name,
                        error: upload_error.to_string(),
                    },
                };

                if !commands.send(next_command) {
                    warn!("Failed to send parsed asset command: flow channel closed");
                }
            });
        }
    }

    async fn parse_uploaded_asset(
        file_name: &str,
        source: AssetUploadSource,
    ) -> Result<ImportedScene> {
        let gltf_loader = GLTFLoader::new();
        match source {
            AssetUploadSource::Bytes(bytes) => {
                gltf_loader
                    .load_from_bytes_with_label(bytes, file_name.to_string())
                    .await
            }
            AssetUploadSource::Bundle(files) => {
                gltf_loader.load_from_file_bundle(file_name, files).await
            }
        }
    }

    pub fn handle_asset_upload_succeeded(
        &self,
        id: String,
        file_name: String,
        diagnostics: Vec<ImportDiagnostic>,
    ) {
        debug!("Successfully loaded asset: {file_name}");
        self.fire_upload_status_success(id, file_name, diagnostics);
    }

    pub fn handle_asset_upload_failed(&self, id: String, file_name: String, error: String) {
        error!("Asset upload failed for `{id}` ({file_name}): {error}");
        self.fire_upload_status_error(id, file_name, error);
    }

    fn send_command(&self, command: FlowCommand) {
        if !self.commands.send(command) {
            warn!("Failed to enqueue flow command: receiver dropped");
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn fire_upload_status_success(
        &self,
        upload_id: String,
        file_name: String,
        diagnostics: Vec<ImportDiagnostic>,
    ) {
        use hyakou_core::types::upload_status::UploadStatusEvent;
        use wasm_bindgen::JsValue;

        let _ = self.upload_status_callback.try_read_shared(|callback| {
            if let Some(callback) = callback {
                let event = UploadStatusEvent::success(upload_id, file_name, diagnostics);
                if let Err(err) = callback.call1(&JsValue::NULL, &event.into()) {
                    warn!("Failed to invoke upload status callback: {err:?}");
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn fire_upload_status_success(
        &self,
        _upload_id: String,
        _file_name: String,
        _diagnostics: Vec<ImportDiagnostic>,
    ) {
    }

    #[cfg(target_arch = "wasm32")]
    fn fire_upload_status_error(&self, upload_id: String, file_name: String, error: String) {
        use hyakou_core::types::upload_status::UploadStatusEvent;
        use wasm_bindgen::JsValue;

        let _ = self.upload_status_callback.try_read_shared(|callback| {
            if let Some(callback) = callback {
                let event = UploadStatusEvent::error(upload_id, file_name, error);
                if let Err(err) = callback.call1(&JsValue::NULL, &event.into()) {
                    warn!("Failed to invoke upload status callback: {err:?}");
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn fire_upload_status_error(&self, _upload_id: String, _file_name: String, _error: String) {}
}
