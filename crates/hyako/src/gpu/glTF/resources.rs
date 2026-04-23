use std::{
    collections::HashMap,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use image::{DynamicImage, ImageFormat};
use log::warn;

use super::ImportContext;
use super::types::ImportedImage;

pub(super) async fn read_asset(path: &Path) -> Result<Vec<u8>> {
    read_bytes(path)
        .await
        .with_context(|| format!("Failed to read glTF asset `{}`", path.display()))
}

pub(super) fn build_uploaded_file_map(
    files: Vec<(String, Vec<u8>)>,
) -> Result<HashMap<String, Vec<u8>>> {
    let mut bundled_files = HashMap::with_capacity(files.len());

    for (name, bytes) in files {
        let normalized_name = normalize_relative_uri(&name)?;
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
    let normalized_name = normalize_relative_uri(file_name).ok()?;
    bundled_files.get(&normalized_name)
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

pub(super) async fn load_images(
    gltf: &gltf::Gltf,
    buffer_data: &[Vec<u8>],
    context: &ImportContext,
) -> Result<Vec<ImportedImage>> {
    let image_async_handles = gltf.images().map(async |image| {
        let image_index = image.index();
        let image_name = image.name().map(str::to_owned);
        let image_result = async {
            let bytes = match image.source() {
                gltf::image::Source::View { view, .. } => {
                    let buffer = buffer_data.get(view.buffer().index()).ok_or_else(|| {
                        anyhow!(
                            "Missing parent buffer {} for image {} in asset `{}`",
                            view.buffer().index(),
                            image.index(),
                            context.asset_label
                        )
                    })?;
                    let start = view.offset();
                    let end = start + view.length();
                    buffer
                        .get(start..end)
                        .ok_or_else(|| {
                            anyhow!(
                                "Image {} in asset `{}` references bytes outside of buffer {}",
                                image.index(),
                                context.asset_label,
                                view.buffer().index()
                            )
                        })?
                        .to_vec()
                }
                gltf::image::Source::Uri { uri, .. } if uri.starts_with("data:") => {
                    gltf::buffer::Data::from_source(gltf::buffer::Source::Uri(uri), None)
                        .map(|data| data.0)
                        .map_err(|error| {
                            anyhow!(
                                "Failed to decode data URI image {} in asset `{}`: {error}",
                                image.index(),
                                context.asset_label
                            )
                        })?
                }
                gltf::image::Source::Uri { uri, .. } => {
                    load_uri_image(uri, image.index(), context).await?
                }
            };

            import_image(image, &bytes, context)
        }
        .await;

        match image_result {
            Ok(imported_image) => imported_image,
            Err(error) => fallback_image_after_warning(image_index, image_name, context, error),
        }
    });

    Ok(futures::future::join_all(image_async_handles).await)
}

fn fallback_image_after_warning(
    image_index: usize,
    image_name: Option<String>,
    context: &ImportContext,
    error: anyhow::Error,
) -> ImportedImage {
    warn!(
        "Falling back to a default texture for image {image_index} in asset `{}` after load/decode failure: {error:?}",
        context.asset_label
    );

    fallback_image(image_index, image_name)
}

fn fallback_image(image_index: usize, image_name: Option<String>) -> ImportedImage {
    ImportedImage {
        index: image_index,
        name: image_name,
        width: 1,
        height: 1,
        pixels_rgba8: vec![255, 255, 255, 255],
    }
}

async fn load_uri_image(uri: &str, image_index: usize, context: &ImportContext) -> Result<Vec<u8>> {
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

async fn load_uri_buffer(
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
        return resolve_uploaded_file(uri, bundled_files)
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
    let normalized_uri = normalize_relative_uri(uri).map_err(|error| {
        anyhow!(
            "Failed to resolve {resource_kind} URI `{uri}` for {resource_kind} {resource_index} in asset `{}`: {error}",
            context.asset_label
        )
    })?;

    if let Some(buffer_base_dir) = context.buffer_base_dir.as_ref() {
        return Ok(buffer_base_dir.join(&normalized_uri));
    }

    if context.bundled_files.is_some() {
        return Ok(PathBuf::from(normalized_uri));
    }

    Err(anyhow!(
        "External {resource_kind} `{uri}` for {resource_kind} {resource_index} in asset `{}` cannot be resolved from in-memory glTF bytes",
        context.asset_label
    ))
}

fn normalize_relative_uri(uri: &str) -> Result<String> {
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

fn import_image(
    image: gltf::Image<'_>,
    encoded_bytes: &[u8],
    context: &ImportContext,
) -> Result<ImportedImage> {
    let decoded_image = decode_image(image.source(), encoded_bytes, image.index(), context)?;
    let rgba8 = decoded_image.to_rgba8();

    Ok(ImportedImage {
        index: image.index(),
        name: image.name().map(str::to_owned),
        width: rgba8.width(),
        height: rgba8.height(),
        pixels_rgba8: rgba8.into_raw(),
    })
}

fn decode_image(
    source: gltf::image::Source<'_>,
    encoded_bytes: &[u8],
    image_index: usize,
    context: &ImportContext,
) -> Result<DynamicImage> {
    let format = match source {
        gltf::image::Source::View { mime_type, .. } => image_format_from_mime_type(mime_type),
        gltf::image::Source::Uri { uri, mime_type } => mime_type
            .and_then(image_format_from_mime_type)
            .or_else(|| image_format_from_uri(uri)),
    }
    .or_else(|| image::guess_format(encoded_bytes).ok())
    .ok_or_else(|| {
        anyhow!(
            "Unsupported image encoding for image {image_index} in asset `{}`",
            context.asset_label
        )
    })?;

    image::load_from_memory_with_format(encoded_bytes, format).with_context(|| {
        format!(
            "Failed to decode image {image_index} in asset `{}`",
            context.asset_label
        )
    })
}

fn image_format_from_mime_type(mime_type: &str) -> Option<ImageFormat> {
    match mime_type {
        "image/png" => Some(ImageFormat::Png),
        "image/jpeg" => Some(ImageFormat::Jpeg),
        _ => None,
    }
}

fn image_format_from_uri(uri: &str) -> Option<ImageFormat> {
    let extension = uri.rsplit('.').next()?.to_ascii_lowercase();

    match extension.as_str() {
        "png" => Some(ImageFormat::Png),
        "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
        _ => None,
    }
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
