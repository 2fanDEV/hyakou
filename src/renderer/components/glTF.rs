use std::{iter::zip, path::Path};

use anyhow::Result;
use nalgebra::{Vector2, Vector3, Vector4};

use crate::renderer::{components::mesh_node::MeshNode, geometry::{mesh::Mesh, vertices::Vertex}};

static BASE_PATH: &str = "assets/gltf";

pub struct GLTFLoader {}

impl GLTFLoader {
    pub fn load_from_path(path: &Path) -> Result<Vec<Mesh>> {
        let slice = std::fs::read(path).unwrap();
        Self::load_from_slice(&slice)
    }

    pub fn load_from_slice(slice: &[u8]) -> Result<Vec<Mesh>> {
        let mut meshes: Vec<Mesh> = vec![];
        let gltf = match gltf::Gltf::from_slice(slice) {
            Ok(gltf) => gltf,
            Err(err) => {
                //todo!();
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
                    std::fs::read(uri).unwrap()
                }
            })
            .collect();

        let gltf_meshes = gltf.meshes().collect::<Vec<gltf::Mesh>>();

        for node in gltf.nodes() {
            let matrix = node.transform().matrix();
            MeshNode::new(mesh, m)
        }

        gltf_meshes.iter().for_each(|mesh| {
            mesh.primitives().for_each(|primitive| {
                let reader = primitive.reader(|buffer| {
                    let index = buffer.index();
                    buffer_data.get(index).map(|data| data.as_slice())
                });
                let positions = reader
                    .read_positions()
                    .unwrap()
                    .map(|vec| Vector3::new(vec[0], vec[1], vec[2]))
                    .collect::<Vec<_>>();
                let indices = reader
                    .read_indices()
                    .unwrap()
                    .into_u32()
                    .collect::<Vec<_>>();
                let normals = reader
                    .read_normals()
                    .unwrap()
                    .map(|vec| Vector3::new(vec[0], vec[1], vec[2]))
                    .collect::<Vec<_>>();
                let tex_coords = reader
                    .read_tex_coords(0)
                    .unwrap()
                    .into_f32()
                    .map(|vec| Vector2::new(vec[0], vec[1]))
                    .collect::<Vec<_>>();
                let gltf_colors = reader.read_colors(0);
                let colors = match gltf_colors {
                    Some(read_colors) => read_colors
                        .into_rgba_f32()
                        .map(|v| Vector4::new(v[0], v[1], v[2], v[3]))
                        .collect::<Vec<_>>(),
                    None => vec![Vector4::new(1.0, 1.0, 1.0, 1.0)],
                };
                let vertices = zip(zip(positions, normals), zip(tex_coords, colors))
                    .map(|((pos, normals), (tex_coords, colors))| {
                        Vertex::new(pos, tex_coords, normals, colors)
                    })
                    .collect::<Vec<_>>();
                meshes.push(Mesh { vertices, indices });
            })
        });
        Ok(meshes)
    }
}
