use std::sync::mpsc::Sender;

use hyakou_core::{
    Shared, SharedAccess, components::LightType, types::import_diagnostic::ImportDiagnostic,
};
use log::{debug, error, warn};

use crate::{flow::RendererCommand, gpu::glTF::ImportedScene, renderer::Renderer};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

pub struct AssetUploadController {
    tx: Sender<RendererCommand>,
    #[cfg(target_arch = "wasm32")]
    upload_status_callback: Shared<Option<js_sys::Function>>,
}

impl AssetUploadController {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(tx: Sender<RendererCommand>) -> Self {
        Self { tx }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(
        tx: Sender<RendererCommand>,
        upload_status_callback: Shared<Option<js_sys::Function>>,
    ) -> Self {
        Self {
            tx,
            upload_status_callback,
        }
    }

    pub fn handle_asset_upload_requested(
        &self,
        id: String,
        file_name: String,
        asset_type: LightType,
        bytes: Vec<u8>,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::gpu::glTF::GLTFLoader;
            let gltf_loader = GLTFLoader::new();
            let parsed_node_graph = pollster::block_on(
                gltf_loader.load_from_bytes_with_label(bytes, file_name.clone()),
            );
            match parsed_node_graph {
                Ok(node_graph) => {
                    self.send_command(RendererCommand::ApplyParsedAsset {
                        id,
                        file_name,
                        asset_type,
                        imported_scene: node_graph,
                    });
                }
                Err(upload_error) => {
                    self.send_command(RendererCommand::AssetUploadFailed {
                        id,
                        file_name,
                        error: upload_error.to_string(),
                    });
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let tx = self.tx.clone();
            spawn_local(async move {
                use crate::gpu::glTF::GLTFLoader;
                let gltf_loader = GLTFLoader::new();
                let parsed_node_graph = gltf_loader
                    .load_from_bytes_with_label(bytes, file_name.clone())
                    .await;
                let next_command = match parsed_node_graph {
                    Ok(node_graph) => RendererCommand::ApplyParsedAsset {
                        id,
                        file_name,
                        asset_type,
                        imported_scene: node_graph,
                    },
                    Err(upload_error) => RendererCommand::AssetUploadFailed {
                        id,
                        file_name,
                        error: upload_error.to_string(),
                    },
                };

                if tx.send(next_command).is_err() {
                    warn!("Failed to send parsed asset command: flow channel closed");
                }
            });
        }
    }

    pub fn handle_asset_bundle_upload_requested(
        &self,
        id: String,
        file_name: String,
        asset_type: LightType,
        files: Vec<(String, Vec<u8>)>,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::gpu::glTF::GLTFLoader;
            let gltf_loader = GLTFLoader::new();
            let parsed_node_graph =
                pollster::block_on(gltf_loader.load_from_file_bundle(&file_name, files));
            match parsed_node_graph {
                Ok(node_graph) => {
                    self.send_command(RendererCommand::ApplyParsedAsset {
                        id,
                        file_name,
                        asset_type,
                        imported_scene: node_graph,
                    });
                }
                Err(upload_error) => {
                    self.send_command(RendererCommand::AssetUploadFailed {
                        id,
                        file_name,
                        error: upload_error.to_string(),
                    });
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let tx = self.tx.clone();
            spawn_local(async move {
                use crate::gpu::glTF::GLTFLoader;
                let gltf_loader = GLTFLoader::new();
                let parsed_node_graph = gltf_loader.load_from_file_bundle(&file_name, files).await;
                let next_command = match parsed_node_graph {
                    Ok(node_graph) => RendererCommand::ApplyParsedAsset {
                        id,
                        file_name,
                        asset_type,
                        imported_scene: node_graph,
                    },
                    Err(upload_error) => RendererCommand::AssetUploadFailed {
                        id,
                        file_name,
                        error: upload_error.to_string(),
                    },
                };

                if tx.send(next_command).is_err() {
                    warn!("Failed to send parsed asset command: flow channel closed");
                }
            });
        }
    }

    pub fn handle_apply_parsed_asset(
        &self,
        renderer_slot: &Shared<Option<Renderer>>,
        id: String,
        file_name: String,
        asset_type: LightType,
        imported_scene: ImportedScene,
    ) {
        let upload_id = id.clone();
        let upload_file_name = file_name.clone();
        let diagnostics = imported_scene.diagnostics.clone();
        let success = renderer_slot
            .try_write_shared(|renderer_slot| {
                let Some(renderer) = renderer_slot.as_mut() else {
                    warn!("Dropping parsed asset `{id}` because renderer is not ready");
                    return false;
                };

                renderer
                    .asset_manager
                    .upload_imported_scene(id, asset_type, imported_scene);
                true
            })
            .unwrap_or(false);

        if success {
            debug!("Successfully loaded asset: {file_name}");
            self.fire_upload_status_success(upload_id, upload_file_name, diagnostics);
        }
    }

    pub fn handle_asset_upload_failed(&self, id: String, file_name: String, error: String) {
        error!("Asset upload failed for `{id}` ({file_name}): {error}");
        self.fire_upload_status_error(id, file_name, error);
    }

    fn send_command(&self, command: RendererCommand) {
        if self.tx.send(command).is_err() {
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
