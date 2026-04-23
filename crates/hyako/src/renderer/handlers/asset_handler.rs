use std::{
    collections::{HashMap, HashSet, hash_set::Iter},
    path::Path,
    rc::Rc,
    sync::Arc,
};

use anyhow::{Result, anyhow};
use glam::Vec4;
use wgpu::{BindGroupLayout, Device, Queue};

use crate::gpu::{
    glTF::{GLTFLoader, ImportedAlphaMode, ImportedMaterial, ImportedScene},
    material::{GpuMaterial, default_sampler_descriptor, sampler_descriptor_from_imported_sampler},
    render_mesh::RenderMesh,
    texture::Texture,
};

use hyakou_core::{
    components::{LightType, mesh_node::MeshNode},
    types::{ModelMatrixBindingMode, ids::MeshId},
};

#[derive(Debug)]
pub struct AssetHandler {
    device: Arc<Device>,
    queue: Queue,
    model_binding_mode: ModelMatrixBindingMode,
    model_bind_group_layout: Option<BindGroupLayout>,
    material_bind_group_layout: BindGroupLayout,
    gltf_loader: GLTFLoader,
    memory_loaded_assets: HashMap<String, Rc<RenderMesh>>,
    visible_assets: HashSet<String>,
}

impl AssetHandler {
    pub fn new(
        device: Arc<Device>,
        queue: Queue,
        model_binding_mode: ModelMatrixBindingMode,
        model_bind_group_layout: Option<BindGroupLayout>,
        material_bind_group_layout: BindGroupLayout,
    ) -> AssetHandler {
        AssetHandler {
            memory_loaded_assets: HashMap::new(),
            gltf_loader: GLTFLoader::new(),
            visible_assets: HashSet::new(),
            device,
            queue,
            model_binding_mode,
            model_bind_group_layout,
            material_bind_group_layout,
        }
    }

    pub async fn upload_from_bytes(
        &mut self,
        id: String,
        light_type: LightType,
        bytes: Vec<u8>,
    ) -> Result<()> {
        let imported_scene = self.gltf_loader.load_from_bytes(bytes).await?;
        self.upload_imported_scene(id, light_type, imported_scene);
        Ok(())
    }

    pub fn upload_imported_scene(
        &mut self,
        id: String,
        light_type: LightType,
        imported_scene: ImportedScene,
    ) -> Option<Rc<RenderMesh>> {
        let fallback_texture = Rc::new(Texture::create_color_texture(
            "Fallback Material Texture",
            &self.device,
            &self.queue,
            1,
            1,
            &[255, 255, 255, 255],
            default_sampler_descriptor("Fallback Material Sampler"),
        ));
        let uploaded_textures = self.upload_textures(&imported_scene, fallback_texture.clone());
        let uploaded_materials = self.upload_materials(
            &imported_scene.materials,
            &uploaded_textures,
            fallback_texture.clone(),
        );
        let default_material = Rc::new(GpuMaterial::new(
            &self.device,
            &self.material_bind_group_layout,
            "Default Material",
            &Self::default_imported_material(),
            fallback_texture,
        ));
        let mesh_nodes = imported_scene.node_graph.flatten();

        self.upload_mesh_node_as_asset(
            id,
            light_type,
            mesh_nodes,
            &uploaded_materials,
            &default_material,
        )
    }

    pub async fn add_from_path(
        &mut self,
        id: String,
        light_type: LightType,
        path: &Path,
    ) -> Result<Rc<RenderMesh>> {
        let imported_scene = self.gltf_loader.load_from_path(path).await?;
        self.upload_imported_scene(id, light_type, imported_scene)
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
        materials: &[Rc<GpuMaterial>],
        default_material: &Rc<GpuMaterial>,
    ) -> Option<Rc<RenderMesh>> {
        let base_id = id;
        let mut render_mesh: Option<Rc<RenderMesh>> = None;

        for (idx, node) in mesh_nodes.into_iter().enumerate() {
            let mesh_id = format!("{base_id}_{idx}");
            let material = node
                .material_index
                .and_then(|material_index| materials.get(material_index).cloned())
                .unwrap_or_else(|| default_material.clone());
            let next_mesh = Rc::new(RenderMesh::new(
                &self.device,
                node,
                material,
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

    fn upload_textures(
        &self,
        imported_scene: &ImportedScene,
        fallback_texture: Rc<Texture>,
    ) -> Vec<Rc<Texture>> {
        imported_scene
            .textures
            .iter()
            .map(|texture| {
                let Some(image) = imported_scene.images.get(texture.image_index) else {
                    return fallback_texture.clone();
                };

                let sampler_descriptor = texture
                    .sampler_index
                    .and_then(|sampler_index| imported_scene.samplers.get(sampler_index))
                    .map(|sampler| {
                        sampler_descriptor_from_imported_sampler(
                            sampler,
                            sampler
                                .name
                                .as_deref()
                                .unwrap_or("Imported Texture Sampler"),
                        )
                    })
                    .unwrap_or_else(|| {
                        default_sampler_descriptor("Default Imported Texture Sampler")
                    });

                Rc::new(Texture::create_color_texture(
                    texture.name.as_deref().unwrap_or("Imported Texture"),
                    &self.device,
                    &self.queue,
                    image.width,
                    image.height,
                    &image.pixels_rgba8,
                    sampler_descriptor,
                ))
            })
            .collect()
    }

    fn upload_materials(
        &self,
        imported_materials: &[ImportedMaterial],
        uploaded_textures: &[Rc<Texture>],
        fallback_texture: Rc<Texture>,
    ) -> Vec<Rc<GpuMaterial>> {
        imported_materials
            .iter()
            .map(|material| {
                let texture = material
                    .base_color_texture
                    .and_then(|texture_ref| {
                        uploaded_textures.get(texture_ref.texture_index).cloned()
                    })
                    .unwrap_or_else(|| fallback_texture.clone());

                Rc::new(GpuMaterial::new(
                    &self.device,
                    &self.material_bind_group_layout,
                    material.name.as_deref().unwrap_or("Imported Material"),
                    material,
                    texture,
                ))
            })
            .collect()
    }

    fn default_imported_material() -> ImportedMaterial {
        ImportedMaterial {
            index: usize::MAX,
            name: None,
            base_color_factor: Vec4::ONE,
            base_color_texture: None,
            alpha_mode: ImportedAlphaMode::Opaque,
            alpha_cutoff: None,
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
