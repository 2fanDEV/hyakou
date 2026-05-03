use hyakou_core::types::ids::MeshId;

#[derive(Clone, Copy)]
pub struct SceneFrameInput<'a> {
    pub outlined_mesh_ids: &'a [MeshId],
}
