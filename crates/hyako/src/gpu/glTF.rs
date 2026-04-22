use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use glam::{Vec2, Vec3, Vec4};
use gltf::mesh::Mode;
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
    pub fn new(path: PathBuf) -> Self {
        Self { BASE_PATH: path }
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
                    if primitive.mode() != Mode::Triangles {
                        return Err(anyhow!(
                            "We only support PrimitiveMode Triangle mode. Found Mode was: {:?}",
                            primitive.mode()
                        ));
                    }

                    let base_color_factor = primitive
                        .material()
                        .pbr_metallic_roughness()
                        .base_color_factor();

                    let base_color_factor_vec4 = Vec4::new(
                        base_color_factor[0],
                        base_color_factor[1],
                        base_color_factor[2],
                        base_color_factor[3],
                    );

                    let reader = primitive.reader(|buffer| {
                        let index = buffer.index();
                        buffer_data.get(index).map(|data| data.as_slice())
                    });

                    let positions = match reader.read_positions() {
                        Some(pos) => pos
                            .map(|iter| Vec3::new(iter[0], iter[1], iter[2]))
                            .collect::<Vec<_>>(),
                        None => {
                            return Err(anyhow!("No positions found for node: {:?}", node.index()));
                        }
                    };

                    let vertex_count = positions.len();

                    let indices = match reader.read_indices() {
                        Some(idx) => idx.into_u32().collect::<Vec<_>>(),
                        None => (0..vertex_count)
                            .map(u32::try_from)
                            .collect::<Result<Vec<_>, _>>()?,
                    };

                    let normals = match reader.read_normals() {
                        Some(normal) => normal
                            .map(|iter| Vec3::new(iter[0], iter[1], iter[2]))
                            .collect::<Vec<_>>(),
                        None => {
                            return Err(anyhow!("No normals found for node: {:?}", node.index()));
                        }
                    };

                    let tex_coords = match reader.read_tex_coords(0) {
                        Some(tex_coord) => tex_coord
                            .into_f32()
                            .map(|tx_coords| Vec2::new(tx_coords[0], tx_coords[1]))
                            .collect::<Vec<_>>(),
                        None => {
                            vec![Vec2::ZERO; vertex_count]
                        }
                    };

                    let gltf_colors = reader.read_colors(0);
                    let colors: Vec<Vec4> = match gltf_colors {
                        Some(read_colors) => read_colors
                            .into_rgba_f32()
                            .map(|v| Vec4::new(v[0], v[1], v[2], v[3]))
                            .collect::<Vec<_>>(),
                        None => vec![Vec4::ONE; vertex_count],
                    };

                    let vertices = (0..vertex_count)
                        .map(|i| {
                            Vertex::new(
                                positions[i],
                                tex_coords[i],
                                normals[i],
                                colors[i],
                                base_color_factor_vec4,
                            )
                        })
                        .collect::<Vec<_>>();

                    Ok(Mesh {
                        name: mesh.name().map(|s| s.to_owned()),
                        vertices,
                        indices,
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            for mesh in meshes {
                mesh_nodes.push(MeshNode::new(
                    mesh,
                    Transform::new(translation, rotation, scale),
                ))
            }
        }
        Ok(mesh_nodes)
    }
}
