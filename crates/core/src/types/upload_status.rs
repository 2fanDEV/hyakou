use wasm_bindgen::prelude::wasm_bindgen;

use crate::types::import_diagnostic::ImportDiagnostic;

#[wasm_bindgen(getter_with_clone)]
pub struct UploadStatusEvent {
    #[wasm_bindgen(js_name = uploadId)]
    pub upload_id: String,
    #[wasm_bindgen(js_name = fileName)]
    pub file_name: String,
    pub status: String,
    pub message: Option<String>,
    pub diagnostics: Vec<String>,
}

impl UploadStatusEvent {
    pub fn success(
        upload_id: String,
        file_name: String,
        diagnostics: Vec<ImportDiagnostic>,
    ) -> Self {
        Self {
            upload_id,
            file_name,
            status: "success".to_string(),
            message: None,
            diagnostics: diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.message)
                .collect(),
        }
    }

    pub fn error(upload_id: String, file_name: String, message: String) -> Self {
        Self {
            upload_id,
            file_name,
            status: "error".to_string(),
            message: Some(message),
            diagnostics: Vec::new(),
        }
    }
}
