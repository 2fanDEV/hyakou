use std::{
    collections::{HashMap, HashSet, hash_set::Iter},
    path::Path,
    sync::Arc,
};

use wgpu::Device;

use crate::renderer::{
    components::{LightType, glTF::GLTFLoader, render_mesh::RenderMesh},
    util::Concatable,
};

#[derive(Debug)]
pub struct AssetManager {
    device: Arc<Device>,
    memory_loaded_assets: HashMap<String, RenderMesh>,
    visible_assets: HashSet<String>,
}

impl AssetManager {
    pub fn new(device: Arc<Device>) -> AssetManager {
        AssetManager {
            memory_loaded_assets: HashMap::new(),
            visible_assets: HashSet::new(),
            device,
        }
    }

    pub fn add_from_path(&mut self, mut id: String, light_type: LightType, path: &Path) {
        let mut idx = 0;
        let mesh_nodes = match GLTFLoader::load_from_path(path) {
            Ok(nodes) => nodes,
            Err(_) => panic!("Couldn't find model at path: {:?}", path),
        };
        for node in mesh_nodes {
            let id = if idx.eq(&0) {
                id.concat("_".to_string().concat(&idx.to_string()))
                    .to_string()
            } else {
                id.clone()
            };
            let render_mesh = RenderMesh::new(&self.device, &node, &light_type, Some(id.clone()));
            self.memory_loaded_assets.insert(id.clone(), render_mesh);
            self.visible_assets.insert(id);
            idx += 1;
        }
    }

    pub fn get(&self, id: String) -> &RenderMesh {
        match self.memory_loaded_assets.get(&id) {
            Some(asset) => asset,
            None => {
                panic!("Asset not found!")
            }
        }
    }

    pub fn get_all_loaded_asset_ids(&self) -> Vec<String> {
        self.memory_loaded_assets.clone().into_keys().collect()
    }

    pub fn get_visible_asset_ids(&self) -> Iter<'_, std::string::String> {
        self.visible_assets.iter()
    }

    pub fn toggle_visibility(&mut self, id: String) {
        let asset_id = self.visible_assets.iter().find(|elem| elem.eq(&&id));
        if asset_id.is_some() {
            self.visible_assets.remove(&id);
        } else {
            self.visible_assets.insert(id);
        }
    }

    pub fn get_all_visible_assets_with_modifier(
        &self,
        light_type: &LightType,
    ) -> impl Iterator<Item = &RenderMesh> {
        self.get_visible_asset_ids()
            .map(|id| self.memory_loaded_assets.get(id).unwrap())
            .filter(move |rm| rm.light_type.eq(&light_type))
    }
}
