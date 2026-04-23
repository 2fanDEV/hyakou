use std::path::Path;

use anyhow::{Context, Result, anyhow};
use image::{DynamicImage, ImageFormat};

use super::ImportContext;
use super::types::ImportedImage;

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

pub(super) async fn load_images(
    gltf: &gltf::Gltf,
    buffer_data: &[Vec<u8>],
    context: &ImportContext,
) -> Result<Vec<ImportedImage>> {
    let image_async_handles = gltf.images().map(async |image| {
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
    });

    futures::future::try_join_all(image_async_handles).await
}

async fn load_uri_image(uri: &str, image_index: usize, context: &ImportContext) -> Result<Vec<u8>> {
    if uri.contains(':') {
        return Err(anyhow!(
            "Unsupported image URI scheme `{uri}` for image {image_index} in asset `{}`",
            context.asset_label
        ));
    }

    let Some(image_base_dir) = context.buffer_base_dir.as_ref() else {
        return Err(anyhow!(
            "External image `{uri}` for image {image_index} in asset `{}` cannot be resolved from in-memory glTF bytes",
            context.asset_label
        ));
    };

    let image_path = image_base_dir.join(uri);
    read_bytes(&image_path).await.with_context(|| {
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
