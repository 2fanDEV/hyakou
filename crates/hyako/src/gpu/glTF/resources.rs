use std::path::Path;

use anyhow::{Context, Result, anyhow};

use super::ImportContext;

pub(super) async fn read_asset(path: &Path) -> Result<Vec<u8>> {
    read_bytes(path)
        .await
        .with_context(|| format!("Failed to read glTF asset `{}`", path.display()))
}

pub(super) async fn load_buffers(
    gltf: &gltf::Gltf,
    context: &ImportContext,
) -> Result<Vec<Vec<u8>>> {
    let buffer_async_handles = gltf.buffers().map(async |buffer| {
        let buffer_index = buffer.index();
        let data = match buffer.source() {
            gltf::buffer::Source::Bin => gltf.blob.clone().ok_or_else(|| {
                anyhow!(
                    "Missing embedded GLB blob for buffer {buffer_index} in asset `{}`",
                    context.asset_label
                )
            }),
            gltf::buffer::Source::Uri(uri) if uri.starts_with("data:") => {
                gltf::buffer::Data::from_source(buffer.source(), None)
                    .map(|data| data.0)
                    .map_err(|error| {
                        anyhow!(
                            "Failed to decode data URI buffer {buffer_index} in asset `{}`: {error}",
                            context.asset_label
                        )
                    })
            }
            gltf::buffer::Source::Uri(uri) => load_uri_buffer(uri, buffer_index, context).await,
        }?;

        ensure_buffer_length(buffer_index, buffer.length(), data, context)
    });

    futures::future::try_join_all(buffer_async_handles).await
}

async fn load_uri_buffer(
    uri: &str,
    buffer_index: usize,
    context: &ImportContext,
) -> Result<Vec<u8>> {
    if uri.contains(':') {
        return Err(anyhow!(
            "Unsupported buffer URI scheme `{uri}` for buffer {buffer_index} in asset `{}`",
            context.asset_label
        ));
    }

    let Some(buffer_base_dir) = context.buffer_base_dir.as_ref() else {
        return Err(anyhow!(
            "External buffer `{uri}` for buffer {buffer_index} in asset `{}` cannot be resolved from in-memory glTF bytes",
            context.asset_label
        ));
    };

    let buffer_path = buffer_base_dir.join(uri);
    read_bytes(&buffer_path).await.with_context(|| {
        format!(
            "Failed to load external buffer `{}` for buffer {buffer_index} in asset `{}`",
            buffer_path.display(),
            context.asset_label
        )
    })
}

fn ensure_buffer_length(
    buffer_index: usize,
    expected_length: usize,
    data: Vec<u8>,
    context: &ImportContext,
) -> Result<Vec<u8>> {
    if data.len() < expected_length {
        return Err(anyhow!(
            "Buffer {buffer_index} in asset `{}` is shorter than declared: expected at least {expected_length} bytes, got {}",
            context.asset_label,
            data.len()
        ));
    }

    Ok(data)
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
