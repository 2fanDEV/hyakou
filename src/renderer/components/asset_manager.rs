use std::{collections::{HashMap, HashSet}, path::Path, sync::Arc};

use anyhow::anyhow;
use wgpu::Device;

use crate::renderer::{components::{glTF::GLTFLoader, mesh_node::{self, MeshNode}, render_mesh::RenderMesh}, util::Concatable};

pub struct AssetManager {
    device: Arc<Device>,
    memory_loaded_assets: HashMap<String, RenderMesh>,
    visible_assets: HashSet<String>
}

impl AssetManager {
    pub fn new(device: Arc<Device>) -> AssetManager { 
        AssetManager { 
            memory_loaded_assets: HashMap::new(),
            visible_assets: HashSet::new(),
            device 
        }
    }

    pub fn add_from_path(&mut self, mut id: String, path: &Path)  {
        let mut idx = 0;
        let mesh_nodes = match GLTFLoader::load_from_path(path) {
            Ok(nodes) => nodes,
            Err(_) => panic!("Couldn't find model at path: {:?}", path),
        };
        for node in mesh_nodes {
            let id = id.concat("_".to_string().concat(&idx.to_string())).to_string();
            let render_mesh = RenderMesh::new(&self.device, &node, Some(id.clone()));
            self.memory_loaded_assets.insert(id, render_mesh);
            idx+=1;
        }
    }

    pub fn get(&self, id: String) -> &RenderMesh {
        match self.memory_loaded_assets.get(&id) {
            Some(asset) => asset,
            None => {
                panic!("Asset not found!")
            },
        }
    }

    pub fn get_all_ids(&self) -> Vec<String> {
          self.memory_loaded_assets.clone().into_keys().collect()
    }
}