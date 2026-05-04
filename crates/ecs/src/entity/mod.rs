use shared_types::id::{BaseId, Id};

mod allocator;
mod entity;
pub use entity::Entity;

#[derive(Debug, Default, Hash, PartialEq, Eq)]
pub struct EntityId {
    id: Id,
    index: usize,
    stale: bool,
    version: usize,
}

impl EntityId {
    pub fn new(id: Id, index: usize, version: usize) -> Self {
        Self {
            id,
            index,
            stale: false,
            version,
        }
    }

    pub fn new_uuid(index: usize, version: usize) -> Self {
        Self {
            id: Id::uuid(),
            index,
            stale: false,
            version,
        }
    }
}
