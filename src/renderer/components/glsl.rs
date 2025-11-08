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
        let gltf_buffer = gltf.blob.as_ref().unwrap();
        
        let meshes = gltf.meshes().collect::<Vec<Mesh>>();
        
        Ok(vec![])
    }
}
