use wgpu::{BindGroup, Buffer};

pub struct RenderObject {
    pub id: String,
    pub mvp_buffer: Buffer,
    pub bind_group: BindGroup,
}



