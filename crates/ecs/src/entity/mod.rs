use shared_types::id::{BaseId, Id};

mod allocator;
mod entity;
pub use allocator::EntityAllocator;
pub use entity::Entity;

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone)]
pub struct EntityId {
    id: Id,
    index: usize,
    version: usize,
}

impl EntityId {
    pub fn new(id: Id, index: usize, version: usize) -> Self {
        Self { id, index, version }
    }

    pub fn new_uuid(index: usize, version: usize) -> Self {
        Self {
            id: Id::uuid(),
            index,
            version,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn version(&self) -> usize {
        self.version
    }
}
