use crate::entity::EntityId;

#[derive(Debug, Default)]
pub struct Entity {
    id: EntityId,
}

impl Entity {
    pub fn new(id: EntityId) -> Self {
        Self { id }
    }

    pub fn new_uuid(index: usize, version: usize) -> Self {
        Self {
            id: EntityId::new_uuid(index, version),
        }
    }

    pub fn id(&self) -> &EntityId {
        &self.id
    }
}
