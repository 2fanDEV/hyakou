use std::ops::Deref;

use crate::renderer::types::Id;

pub trait UniformResourceId: Id {
    fn get(&self) -> &str;
}

#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Default, Debug, Eq, PartialEq, Clone, Hash)]
pub struct MeshId(pub String);

impl Deref for MeshId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Id for MeshId {
    fn get_id(&self) -> &str {
        &self.0
    }
}
