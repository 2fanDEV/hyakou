use std::{
    collections::{HashMap, HashSet, hash_set::Iter},
    path::Path,
    rc::Rc,
    sync::Arc,
};

use wgpu::Device;

use crate::renderer::{
    components::{LightType, glTF::GLTFLoader, render_mesh::RenderMesh},
    util::{self, Concatable},
};

#[derive(Debug)]
pub struct AssetHandler {
    device: Arc<Device>,
    gltf_loader: GLTFLoader,
    memory_loaded_assets: HashMap<String, Rc<RenderMesh>>,
    visible_assets: HashSet<String>,
}

impl AssetHandler {
    pub fn new(device: Arc<Device>) -> AssetHandler {
        AssetHandler {
            memory_loaded_assets: HashMap::new(),
            gltf_loader: GLTFLoader::new(util::get_relative_path()),
            visible_assets: HashSet::new(),
            device,
        }
    }

    pub fn add_from_path(
        &mut self,
        mut id: String,
        light_type: LightType,
        path: &Path,
    ) -> Option<Rc<RenderMesh>> {
        //TODO make rendermesh be a node consisting of multiple nodes
        let mut idx = 0;
        let mesh_nodes = match self.gltf_loader.load_from_path(path) {
            Ok(nodes) => nodes,
            Err(_) => panic!("Couldn't find model at path: {:?}", path),
        };
        let mut render_mesh = None;
        for node in mesh_nodes {
            let id = if idx.eq(&0) {
                id.concat("_".to_string().concat(&idx.to_string()))
                    .to_string()
            } else {
                id.clone()
            };
            render_mesh = Some(Rc::new(RenderMesh::new(
                &self.device,
                node,
                &light_type,
                Some(id.clone()),
            )));
            self.memory_loaded_assets
                .insert(id.clone(), render_mesh.as_ref().unwrap().clone());
            self.visible_assets.insert(id);
            idx += 1;
        }
        render_mesh
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
        &mut self,
        light_type: &LightType,
    ) -> impl Iterator<Item = &Rc<RenderMesh>> {
        self.get_visible_asset_ids()
            .map(|id| self.memory_loaded_assets.get(id).unwrap())
            .filter(move |rm| rm.light_type.eq(&light_type))
    }

    pub fn get_visible_asset_by_id(&mut self, id: &str) -> &mut Rc<RenderMesh> {
        self.memory_loaded_assets.get_mut(id).unwrap()
    }
}
