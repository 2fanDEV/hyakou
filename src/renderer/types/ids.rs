use crate::renderer::types::Id;

pub trait UniformResourceId: Id {
    fn get(&self) -> &str;
}

#[derive(Clone)]
pub struct UniformBufferId {
    pub id: String,
}

impl UniformBufferId {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl Id for UniformBufferId {
    fn get_id(&self) -> &str {
        &self.id
    }
}
impl UniformResourceId for UniformBufferId {
    fn get(&self) -> &str {
        self.get_id()
    }
}
