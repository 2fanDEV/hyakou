use std::{iter::zip, path::Path};

use anyhow::Result;
use glam::{Vec2, Vec3, Vec4};

use crate::renderer::{
    components::{mesh_node::MeshNode, render_mesh::Transform},
    geometry::{mesh::Mesh, vertices::Vertex},
};

static BASE_PATH: &str = "/Users/zapzap/Projects/hyakou/assets/gltf";

pub struct GLTFLoader {}

impl GLTFLoader {
    pub fn load_from_path(path: &Path) -> Result<Vec<MeshNode>> {
        let slice = std::fs::read(path).unwrap();
        Self::load_from_slice(slice)
    }

    pub fn load_from_slice(slice: Vec<u8>) -> Result<Vec<MeshNode>> {
        let mut mesh_nodes: Vec<MeshNode> = vec![];
        let gltf = match gltf::Gltf::from_slice(&slice) {
            Ok(gltf) => gltf,
            Err(_) => {
                //TODO: Better error message;
                panic!("ERROR while parsing gltf/glb");
            }
        };
        let buffer_data: Vec<Vec<u8>> = gltf
            .buffers()
            .map(|buffer| match buffer.source() {
                gltf::buffer::Source::Bin => gltf.blob.clone().unwrap(),
                gltf::buffer::Source::Uri(uri) => {
                    let base_path = Path::new(BASE_PATH);
                    let uri = base_path.join(uri);
                    println!("{:?}", uri);
                    std::fs::read(uri).unwrap()
                }
            })
            .collect();

        for node in gltf.nodes() {
            let (translation, rotation, scale) = node.transform().decomposed();
            let translation = Vec3::new(translation[0], translation[1], translation[2]);
            let rotation = glam::Quat::from_array(rotation).normalize();
            let scale = Vec3::new(scale[0], scale[1], scale[2]);
            let matrix = node.transform().matrix();
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
                    Transform {
                        position: translation,
                        rotation,
                        scale,
                    },
                ))
            });
        }
        Ok(mesh_nodes)
    }
}
