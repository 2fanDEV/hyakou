use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

use glam::Vec4;
use wgpu::{BindGroupLayout, Device, Queue};

use crate::{
    gpu::{
        glTF::{ImportedAlphaMode, ImportedMaterial, ImportedScene},
        material::{
            GpuMaterial, default_sampler_descriptor, sampler_descriptor_from_imported_sampler,
        },
        render_mesh::RenderMesh,
        texture::Texture,
    },
    renderer::renderer_context::RenderContext,
};

use hyakou_core::{
    components::{AssetType, mesh_node::MeshNode},
    geometry::ray::{Ray, math::intersect_transformed_mesh},
    selection::structure::{SelectionScope, SelectionTarget},
    types::{ModelMatrixBindingMode, ids::MeshId},
};
use shared::SharedAccess;

#[derive(Debug)]
pub struct AssetHandler {
    device: Arc<Device>,
    queue: Arc<Queue>,
    model_binding_mode: ModelMatrixBindingMode,
    model_bind_group_layout: Option<BindGroupLayout>,
    material_bind_group_layout: BindGroupLayout,
    memory_loaded_assets: HashMap<String, Rc<RenderMesh>>,
    visible_assets: HashSet<String>,
    asset_groups: HashMap<String, Vec<MeshId>>,
    mesh_asset_groups: HashMap<MeshId, String>,
}

impl AssetHandler {
    fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        model_binding_mode: ModelMatrixBindingMode,
        model_bind_group_layout: Option<BindGroupLayout>,
        material_bind_group_layout: BindGroupLayout,
    ) -> AssetHandler {
        AssetHandler {
            memory_loaded_assets: HashMap::new(),
            visible_assets: HashSet::new(),
            asset_groups: HashMap::new(),
            mesh_asset_groups: HashMap::new(),
            device,
            queue,
            model_binding_mode,
            model_bind_group_layout,
            material_bind_group_layout,
        }
    }

    pub(crate) fn from_render_context(ctx: &RenderContext) -> Self {
        Self::new(
            ctx.device.clone(),
            ctx.queue.clone(),
            ctx.model_binding_mode,
            ctx.model_bind_group_layout.clone(),
            ctx.material_bind_group_layout.clone(),
        )
    }

    pub(crate) fn upload_imported_scene(
        &mut self,
        id: String,
        asset_type: AssetType,
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
            asset_type,
            mesh_nodes,
            &uploaded_materials,
            &default_material,
        )
    }

    fn upload_mesh_node_as_asset(
        &mut self,
        id: String,
        asset_type: AssetType,
        mesh_nodes: Vec<MeshNode>,
        materials: &[Rc<GpuMaterial>],
        default_material: &Rc<GpuMaterial>,
    ) -> Option<Rc<RenderMesh>> {
        let base_id = id;
        let mut render_mesh: Option<Rc<RenderMesh>> = None;
        let mut group_mesh_ids = Vec::new();

        for (idx, node) in mesh_nodes.into_iter().enumerate() {
            let mesh_id = format!("{base_id}_{idx}");
            let mesh_id = MeshId(mesh_id);
            let material = node
                .material_index
                .and_then(|material_index| materials.get(material_index).cloned())
                .unwrap_or_else(|| default_material.clone());
            let next_mesh = Rc::new(RenderMesh::new(
                &self.device,
                node,
                material,
                &asset_type,
                Some(mesh_id.clone()),
                self.model_binding_mode,
                self.model_bind_group_layout.as_ref(),
            ));
            self.memory_loaded_assets
                .insert(mesh_id.0.clone(), next_mesh.clone());
            self.visible_assets.insert(mesh_id.0.clone());
            self.mesh_asset_groups
                .insert(mesh_id.clone(), base_id.clone());
            group_mesh_ids.push(mesh_id);
            render_mesh = Some(next_mesh);
        }

        if !group_mesh_ids.is_empty() {
            self.asset_groups.insert(base_id, group_mesh_ids);
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

    fn visible_asset_ids(&self) -> impl Iterator<Item = &String> {
        self.visible_assets.iter()
    }

    fn get_all_visible_assets(&self) -> impl Iterator<Item = &Rc<RenderMesh>> {
        self.visible_asset_ids()
            .filter_map(|id| self.memory_loaded_assets.get(id))
    }

    pub(crate) fn get_visible_asset(&self, id: &MeshId) -> Option<&Rc<RenderMesh>> {
        if !self.visible_assets.contains(&id.0) {
            return None;
        }

        self.memory_loaded_assets.get(&id.0)
    }

    fn selection_ids_for(&self, hit_mesh_id: &MeshId, scope: &SelectionScope) -> Vec<MeshId> {
        match scope {
            SelectionScope::Node => vec![hit_mesh_id.clone()],
            SelectionScope::Object => self
                .mesh_asset_groups
                .get(hit_mesh_id)
                .and_then(|group_id| self.asset_groups.get(group_id))
                .cloned()
                .unwrap_or_else(|| vec![hit_mesh_id.clone()]),
        }
    }

    fn ray_cast(&self, ray: &Ray) -> Option<MeshId> {
        let mut closest_hit: Option<(MeshId, f32)> = None;

        for render_mesh in self.get_all_visible_assets() {
            let hit = render_mesh
                .transform
                .try_read_shared(|transform| {
                    intersect_transformed_mesh(ray, &render_mesh.mesh, transform)
                })
                .ok()
                .flatten();

            let Some(hit) = hit else {
                continue;
            };

            if closest_hit
                .as_ref()
                .is_none_or(|(_, distance)| hit.distance < *distance)
            {
                closest_hit = Some((render_mesh.id.clone(), hit.distance));
            }
        }

        closest_hit.map(|(mesh_id, _)| mesh_id)
    }

    pub(crate) fn resolve_selection_target(
        &self,
        ray: &Ray,
        scope: SelectionScope,
    ) -> Option<SelectionTarget> {
        let hit_mesh_id = self.ray_cast(ray)?;
        let outline_mesh_ids = self.selection_ids_for(&hit_mesh_id, &scope);

        Some(SelectionTarget::new(hit_mesh_id, outline_mesh_ids, scope))
    }

    pub(crate) fn visible_meshes_with_asset_type(
        &mut self,
        asset_type: &AssetType,
    ) -> impl Iterator<Item = &Rc<RenderMesh>> {
        self.visible_asset_ids()
            .map(|id| self.memory_loaded_assets.get(id).unwrap())
            .filter(move |rm| rm.light_type.eq(asset_type))
    }
}
