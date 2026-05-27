use std::ops::Deref;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UniformBufferId {
    pub id: String,
}

impl UniformBufferId {
    pub fn new(id: String) -> Self {
        Self { id }
    }

    pub fn as_str(&self) -> &str {
        &self.id
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
