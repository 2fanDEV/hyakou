use std::{
    collections::{HashMap, HashSet, hash_set::Iter},
    path::Path,
    rc::Rc,
    sync::Arc,
};

use anyhow::Result;
use wgpu::BindGroupLayout;
use wgpu::Device;

use crate::{
    gpu::{glTF::GLTFLoader, render_mesh::RenderMesh},
    renderer::util::{self, Concatable},
};

use hyakou_core::{
    components::{LightType, mesh_node::MeshNode},
    types::{ModelMatrixBindingMode, ids::MeshId},
};

#[derive(Debug)]
pub struct AssetHandler {
    device: Arc<Device>,
    model_binding_mode: ModelMatrixBindingMode,
    model_bind_group_layout: Option<BindGroupLayout>,
    gltf_loader: GLTFLoader,
    memory_loaded_assets: HashMap<String, Rc<RenderMesh>>,
    visible_assets: HashSet<String>,
}

impl AssetHandler {
    pub fn new(
        device: Arc<Device>,
        model_binding_mode: ModelMatrixBindingMode,
        model_bind_group_layout: Option<BindGroupLayout>,
    ) -> AssetHandler {
        AssetHandler {
            memory_loaded_assets: HashMap::new(),
            gltf_loader: GLTFLoader::new(util::get_relative_path()),
            visible_assets: HashSet::new(),
            device,
            model_binding_mode,
            model_bind_group_layout,
        }
    }

    pub async fn upload_from_bytes(
        &mut self,
        id: String,
        light_type: LightType,
        bytes: Vec<u8>,
    ) -> Result<()> {
        let mesh_nodes = self.gltf_loader.load_from_bytes(bytes).await?;
        self.upload_mesh_node_as_asset(id, light_type, mesh_nodes);
        Ok(())
    }

    pub fn upload_mesh_nodes(
        &mut self,
        id: String,
        light_type: LightType,
        mesh_nodes: Vec<MeshNode>,
    ) -> Option<Rc<RenderMesh>> {
        self.upload_mesh_node_as_asset(id, light_type, mesh_nodes)
    }

    pub async fn add_from_path(
        &mut self,
        id: String,
        light_type: LightType,
        path: &Path,
    ) -> Option<Rc<RenderMesh>> {
        //TODO make rendermesh be a node consisting of multiple nodes
        let mesh_nodes = match self.gltf_loader.load_from_path(path).await {
            Ok(nodes) => nodes,
            Err(_) => panic!("Couldn't find model at path: {:?}", path),
        };
        self.upload_mesh_node_as_asset(id, light_type, mesh_nodes)
    }

    fn upload_mesh_node_as_asset(
        &mut self,
        mut id: String,
        light_type: LightType,
        mesh_nodes: Vec<MeshNode>,
    ) -> Option<Rc<RenderMesh>> {
        let mut idx = 0;
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
                Some(MeshId(id.clone())),
                self.model_binding_mode,
                self.model_bind_group_layout.as_ref(),
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
