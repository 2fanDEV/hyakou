use shared_types::id::{BaseId, Id};

mod allocator;
mod entity;
pub use allocator::EntityAllocator;
pub use entity::Entity;

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone)]
pub struct EntityId {
    id: Id,
    index: usize,
<<<<<<< HEAD
=======
    stale: bool,
>>>>>>> f5ec7962a6d6e69dec83a04462e5d86534679c64
    version: usize,
}

impl EntityId {
    pub fn new(id: Id, index: usize, version: usize) -> Self {
<<<<<<< HEAD
        Self { id, index, version }
=======
        Self {
            id,
            index,
            stale: false,
            version,
        }
>>>>>>> f5ec7962a6d6e69dec83a04462e5d86534679c64
    }

    pub fn new_uuid(index: usize, version: usize) -> Self {
        Self {
            id: Id::uuid(),
            index,
<<<<<<< HEAD
            version,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn version(&self) -> usize {
        self.version
    }
=======
            stale: false,
            version,
        }
    }
>>>>>>> f5ec7962a6d6e69dec83a04462e5d86534679c64
}
