use std::{
    collections::{HashMap, HashSet, hash_set::Iter},
    path::Path,
    rc::Rc,
    sync::Arc,
};

use anyhow::{Result, anyhow};
use wgpu::BindGroupLayout;
use wgpu::Device;

use crate::gpu::{glTF::GLTFLoader, render_mesh::RenderMesh};

use hyakou_core::{
    components::{LightType, mesh_node::MeshNode},
    geometry::node::NodeGraph,
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
            gltf_loader: GLTFLoader::new(),
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
        let node_graph = self.gltf_loader.load_from_bytes(bytes).await?;
        self.upload_node_graph(id, light_type, node_graph);
        Ok(())
    }

    pub fn upload_node_graph(
        &mut self,
        id: String,
        light_type: LightType,
        node_graph: NodeGraph,
    ) -> Option<Rc<RenderMesh>> {
        let mesh_nodes = node_graph.flatten();
        self.upload_mesh_node_as_asset(id, light_type, mesh_nodes)
    }

    pub async fn add_from_path(
        &mut self,
        id: String,
        light_type: LightType,
        path: &Path,
    ) -> Result<Rc<RenderMesh>> {
        //TODO make rendermesh be a node consisting of multiple nodes
        let node_graph = self.gltf_loader.load_from_path(path).await?;
        self.upload_node_graph(id, light_type, node_graph)
            .ok_or_else(|| {
                anyhow!(
                    "glTF asset `{}` produced no renderable meshes",
                    path.display()
                )
            })
    }

    fn upload_mesh_node_as_asset(
        &mut self,
        id: String,
        light_type: LightType,
        mesh_nodes: Vec<MeshNode>,
    ) -> Option<Rc<RenderMesh>> {
        let base_id = id;
        let mut render_mesh: Option<Rc<RenderMesh>> = None;
        for (idx, node) in mesh_nodes.into_iter().enumerate() {
            let mesh_id = format!("{base_id}_{idx}");
            let next_mesh = Rc::new(RenderMesh::new(
                &self.device,
                node,
                &light_type,
                Some(MeshId(mesh_id.clone())),
                self.model_binding_mode,
                self.model_bind_group_layout.as_ref(),
            ));
            self.memory_loaded_assets
                .insert(mesh_id.clone(), next_mesh.clone());
            self.visible_assets.insert(mesh_id);
            render_mesh = Some(next_mesh);
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
