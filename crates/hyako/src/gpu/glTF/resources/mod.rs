use std::collections::HashMap;

use anyhow::{Result, anyhow};

mod loader;
mod path;

pub(super) use loader::{load_buffers, load_images};
pub(super) use path::read_asset;

pub(super) fn build_uploaded_file_map(
    files: Vec<(String, Vec<u8>)>,
) -> Result<HashMap<String, Vec<u8>>> {
    let mut bundled_files = HashMap::with_capacity(files.len());

    for (name, bytes) in files {
        let normalized_name = path::normalize_relative_uri(&name)?;
        if bundled_files
            .insert(normalized_name.clone(), bytes)
            .is_some()
        {
            return Err(anyhow!(
                "Duplicate uploaded resource `{normalized_name}` in glTF bundle"
            ));
        }
    }

    Ok(bundled_files)
}

pub(super) fn resolve_uploaded_file<'a>(
    file_name: &str,
    bundled_files: &'a HashMap<String, Vec<u8>>,
) -> Option<&'a Vec<u8>> {
    let normalized_name = path::normalize_relative_uri(file_name).ok()?;
    bundled_files.get(&normalized_name)
}
