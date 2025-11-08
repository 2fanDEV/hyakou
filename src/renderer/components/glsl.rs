use std::fs::read;

use anyhow::Result;
use gltf::{Gltf, Mesh};

use crate::renderer::geometry::vertices::Vertex;

pub struct GLTFLoader {}

impl GLTFLoader {
    pub fn load_from_path() {}

    pub fn load_from_slice(slice: &[u8]) -> Result<Vec<Vertex>> {
        let mut vertices: Vec<Vertex> = vec![];
        let gltf = match gltf::Gltf::from_slice(slice) {
            Ok(gltf) => gltf,
            Err(err) => {
                //todo!();
                panic!("ERROR while parsing gltf/glb");
            }
        };
        
        let meshes = gltf.meshes().collect::<Vec<Mesh>>();
        meshes.iter().enumerate().map(|(idx, mesh) {
            mesh.primitives().enumerate().map(|(p_idx, primitive)| {
                let reader = primitive.reader(|buffer| buffer);
                const POSITIONS: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
                const NORMALS: Vec<[f32; 3]> = reader.read_normals().unwrap().collect();
                let colors = reader.read_colors(p_idx).unwrap().into_rgba_f32().collect::<Vec<_>>();
            })
        });
        Ok(vec![])
    }
}
