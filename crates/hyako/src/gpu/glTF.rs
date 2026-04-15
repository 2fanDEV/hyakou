use std::{
    iter::zip,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use glam::{Vec2, Vec3, Vec4};
use hyakou_core::{
    components::mesh_node::MeshNode,
    geometry::{mesh::Mesh, vertices::Vertex},
    types::transform::Transform,
};

use crate::renderer::util::Concatable;

#[derive(Debug, Clone)]
pub struct GLTFLoader {
    BASE_PATH: PathBuf,
}

impl GLTFLoader {
    pub fn new(p: PathBuf) -> Self {
        Self { BASE_PATH: p }
    }

    #[cfg(target_arch = "wasm32")]
    async fn read_bytes(&self, path: &Path) -> Result<Vec<u8>> {
        use gloo_net::http::Request;
        let slice = Request::get(path.to_str().unwrap()).build()?;
        let response = slice.send().await?;
        let slice = match response.binary().await {
            Ok(s) => s,
            Err(e) => return Err(anyhow!(e)),
        };
        Ok(slice)
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn read_bytes(&self, path: &Path) -> Result<Vec<u8>> {
        let slice = std::fs::read(path).unwrap();
        Ok(slice)
    }

    pub async fn load_from_path(&self, path: &Path) -> Result<Vec<MeshNode>> {
        let slice = match self.read_bytes(path).await {
            Ok(slice) => slice,
            Err(e) => return Err(e),
        };
        self.load_from_bytes(slice).await
    }

    pub async fn load_from_bytes(&self, slice: Vec<u8>) -> Result<Vec<MeshNode>> {
        let mut mesh_nodes: Vec<MeshNode> = vec![];
        let gltf = match gltf::Gltf::from_slice(&slice) {
            Ok(gltf) => gltf,
            Err(_) => {
                return Err(anyhow!(
                    "The given slice does not contain any mesh or gltf object!"
                ));
            }
        };
        let buffer_async_handles = gltf.buffers().map(async |buffer| match buffer.source() {
            gltf::buffer::Source::Bin => gltf
                .blob
                .clone()
                .ok_or_else(|| anyhow!("Glb blob is missing")),
            gltf::buffer::Source::Uri(uri) => {
                let base_path = Path::new(&self.BASE_PATH);
                let uri = base_path.join("assets/gltf/".to_string().concat(uri));
                println!("{:?}", uri);
                self.read_bytes(&uri).await
            }
        });

        let buffer_data = futures::future::try_join_all(buffer_async_handles)
            .await
            .expect("Not able to join all async handles for gltf file");

        for node in gltf.nodes() {
            let (translation, rotation, scale) = node.transform().decomposed();
            let translation = Vec3::new(translation[0], translation[1], translation[2]);
            let rotation = glam::Quat::from_array(rotation).normalize();
            let scale = Vec3::new(scale[0], scale[1], scale[2]);
            let mesh = match node.mesh() {
                Some(mesh) => mesh,
                None => continue,
            };

            let meshes = mesh
                .primitives()
                .map(|primitive| {
                    let reader = primitive.reader(|buffer| {
                        let index = buffer.index();
                        buffer_data.get(index).map(|data| data.as_slice())
                    });

                    let positions = reader
                        .read_positions()
                        .unwrap()
                        .map(|vec| Vec3::new(vec[0], vec[1], vec[2]))
                        .collect::<Vec<_>>();

                    let indices = reader
                        .read_indices()
                        .unwrap()
                        .into_u32()
                        .collect::<Vec<_>>();

                    let normals = reader
                        .read_normals()
                        .unwrap()
                        .map(|vec| Vec3::new(vec[0], vec[1], vec[2]))
                        .collect::<Vec<_>>();

                    let tex_coords = reader
                        .read_tex_coords(0)
                        .unwrap()
                        .into_f32()
                        .map(|vec| Vec2::new(vec[0], vec[1]))
                        .collect::<Vec<_>>();

                    let gltf_colors = reader.read_colors(0);

                    let colors: Vec<Vec4> = match gltf_colors {
                        Some(read_colors) => read_colors
                            .into_rgba_f32()
                            .map(|v| Vec4::new(v[0], v[1], v[2], v[3]))
                            .collect::<Vec<_>>(),
                        None => vec![Vec4::new(0.0, 0.0, 0.0, 0.0)],
                    };

                    let vertices = zip(zip(positions, normals), tex_coords)
                        .map(|((pos, normals), tex_coords)| {
                            Vertex::new(pos, tex_coords, normals, colors[0])
                        })
                        .collect::<Vec<_>>();

                    Mesh {
                        name: mesh.name().map(|s| s.to_owned()),
                        vertices,
                        indices,
                    }
                })
                .collect::<Vec<_>>();
            meshes.into_iter().for_each(|mesh| {
                mesh_nodes.push(MeshNode::new(
                    mesh,
                    Transform::new(translation, rotation, scale),
                ))
            });
        }
        Ok(mesh_nodes)
    }
}
