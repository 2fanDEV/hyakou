use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, anyhow};

use crate::gpu::glTF::ImportContext;

pub(in crate::gpu::glTF) async fn read_asset(path: &Path) -> Result<Vec<u8>> {
    read_bytes(path)
        .await
        .with_context(|| format!("Failed to read glTF asset `{}`", path.display()))
}

pub(super) async fn load_uri_image(
    uri: &str,
    image_index: usize,
    context: &ImportContext,
) -> Result<Vec<u8>> {
    let image_path = resolve_external_resource_path(uri, context, "image", image_index)?;
    let image_bytes = read_resource_bytes(uri, &image_path, context).await;

    image_bytes.with_context(|| {
        format!(
            "Failed to load external image `{}` for image {image_index} in asset `{}`",
            image_path.display(),
            context.asset_label
        )
    })
}

pub(super) async fn load_uri_buffer(
    uri: &str,
    buffer_index: usize,
    context: &ImportContext,
) -> Result<Vec<u8>> {
    let buffer_path = resolve_external_resource_path(uri, context, "buffer", buffer_index)?;
    let buffer_bytes = read_resource_bytes(uri, &buffer_path, context).await;

    buffer_bytes.with_context(|| {
        format!(
            "Failed to load external buffer `{}` for buffer {buffer_index} in asset `{}`",
            buffer_path.display(),
            context.asset_label
        )
    })
}

async fn read_resource_bytes(
    uri: &str,
    resolved_path: &Path,
    context: &ImportContext,
) -> Result<Vec<u8>> {
    if let Some(bundled_files) = context.bundled_files.as_ref() {
        return super::resolve_uploaded_file(uri, bundled_files)
            .cloned()
            .ok_or_else(|| anyhow!("Missing uploaded sidecar resource `{uri}`"));
    }

    read_bytes(resolved_path).await
}

fn resolve_external_resource_path(
    uri: &str,
    context: &ImportContext,
    resource_kind: &str,
    resource_index: usize,
) -> Result<PathBuf> {
    if context.bundled_files.is_some() {
        let normalized_uri = normalize_relative_uri(uri).map_err(|error| {
            anyhow!(
                "Failed to resolve {resource_kind} URI `{uri}` for {resource_kind} {resource_index} in asset `{}`: {error}",
                context.asset_label
            )
        })?;

        return Ok(PathBuf::from(normalized_uri));
    }

    if uri.contains(':') {
        return Err(anyhow!(
            "Unsupported {resource_kind} URI scheme `{uri}` for {resource_kind} {resource_index} in asset `{}`",
            context.asset_label
        ));
    }

    if let Some(buffer_base_dir) = context.buffer_base_dir.as_ref() {
        return Ok(buffer_base_dir.join(uri));
    }

    Err(anyhow!(
        "External {resource_kind} `{uri}` for {resource_kind} {resource_index} in asset `{}` cannot be resolved from in-memory glTF bytes",
        context.asset_label
    ))
}

pub(super) fn normalize_relative_uri(uri: &str) -> Result<String> {
    if uri.is_empty() {
        return Err(anyhow!("relative resource path cannot be empty"));
    }

    if uri.contains(':') {
        return Err(anyhow!("unsupported URI scheme"));
    }

    let path = Path::new(uri);
    if path.is_absolute() {
        return Err(anyhow!("absolute paths are not allowed"));
    }

    let mut normalized_components = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                normalized_components.push(part.to_string_lossy().into_owned())
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if normalized_components.pop().is_none() {
                    return Err(anyhow!("path escapes the uploaded bundle"));
                }
            }
            Component::Prefix(_) | Component::RootDir => {
                return Err(anyhow!("absolute paths are not allowed"));
            }
        }
    }

    if normalized_components.is_empty() {
        return Err(anyhow!("relative resource path cannot be empty"));
    }

    Ok(normalized_components.join("/"))
}

#[cfg(target_arch = "wasm32")]
async fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    use gloo_net::http::Request;

    let path = path
        .to_str()
        .ok_or_else(|| anyhow!("Path is not valid UTF-8: {}", path.display()))?;
    let request = Request::get(path)
        .build()
        .with_context(|| format!("Failed to build request for glTF resource `{path}`"))?;
    let response = request
        .send()
        .await
        .with_context(|| format!("Failed to fetch glTF resource `{path}`"))?;

    if !response.ok() {
        return Err(anyhow!(
            "Failed to fetch glTF resource `{path}`: HTTP {}",
            response.status()
        ));
    }

    response
        .binary()
        .await
        .with_context(|| format!("Failed to read glTF resource bytes from `{path}`"))
}

#[cfg(not(target_arch = "wasm32"))]
async fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path)
        .map_err(|error| anyhow!("Failed to read glTF resource `{}`: {error}", path.display()))
}
