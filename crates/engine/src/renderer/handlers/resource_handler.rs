use std::collections::HashMap;

use anyhow::Result;
use wgpu::BindGroup;

use crate::renderer::types::ids::UniformResourceId;

#[derive(Default)]
pub struct ResourceHandler {
    resource_map: HashMap<String, BindGroup>,
}

impl ResourceHandler {
    pub fn insert(&mut self, id: Box<dyn UniformResourceId>, bind_group: BindGroup) -> Result<()> {
        self.resource_map.insert(id.get().to_owned(), bind_group);
        Ok(())
    }
}
