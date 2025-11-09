use std::iter::zip;

use anyhow::Result;
use nalgebra::{Vector2, Vector3, Vector4};

use crate::renderer::geometry::{mesh::Mesh, vertices::Vertex};

pub struct GLTFLoader {}

impl GLTFLoader {
    pub fn load_from_path() {}

    pub fn load_from_slice(slice: &[u8]) -> Result<Vec<Mesh>> {
        let mut meshes: Vec<Mesh> = vec![];
        let gltf = match gltf::Gltf::from_slice(slice) {
            Ok(gltf) => gltf,
            Err(err) => {
                //todo!();
                panic!("ERROR while parsing gltf/glb");
            }
        };
        
        let buffer_data: Vec<Vec<u8>> = gltf.buffers()
            .map(|buffer| match buffer.source() {
                gltf::buffer::Source::Bin => gltf.blob.clone().unwrap(),
                gltf::buffer::Source::Uri(uri) => std::fs::read(uri).unwrap(),
            })
            .collect();
        
        let gltf_meshes = gltf.meshes().collect::<Vec<gltf::Mesh>>();
        gltf_meshes.iter().enumerate().for_each(|(idx, mesh)| {
            mesh.primitives().enumerate().for_each(|(p_idx, primitive)| {
                let reader = primitive.reader(|buffer| {
                    let index = buffer.index();
                    buffer_data.get(index).map(|data| data.as_slice())
                });
                let positions = reader.read_positions().unwrap().map(|vec| {
                    Vector3::new(vec[0], vec[1], vec[2])
                }).collect::<Vec<_>>();
                let normals = reader.read_normals().unwrap()
                .map(|vec|
                    Vector3::new(vec[0], vec[1], vec[2])
                 ).collect::<Vec<_>>();
                let tex_coords = reader.read_tex_coords(0).unwrap().into_f32()
                .map(|vec|
                    Vector2::new(vec[0], vec[1]))
                     .collect::<Vec<_>>();
                let colors = reader.read_colors(0).unwrap().into_rgba_f32()
                .map(|vec| Vector4::new(vec[0], vec[1], vec[2], vec[3])).collect::<Vec<_>>();
                let vertices = zip(
                 zip(positions, normals),
                 zip(tex_coords, colors)
                ).map(|(
                 (pos, normals),
                 (tex_coords, colors)
                )| 
                 Vertex::new(pos, tex_coords, normals, colors))
                .collect::<Vec<_>>();

                meshes.push(Mesh {
                    vertices,
                });
            })
        });
        Ok(vec![])
    }
}
