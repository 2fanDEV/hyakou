use crate::{Component, EntityAllocator, Events};

/// Owns ECS state for one isolated world.
///
/// `World` APIs should expose stable ECS operations only. Immediate operations mutate world-owned
/// state directly; deferred operations are recorded into command buffers and applied later.
/// Invalid entity operations must fail safely instead of panicking. Missing component or resource
/// access should return absence-style results when those access APIs are introduced.
#[derive(Debug)]
pub struct World {
    components: Vec<Box<dyn Component>>,
    allocator: EntityAllocator,
    events: Events,
}

impl World {
    pub fn new(
        components: Vec<Box<dyn Component>>,
        allocator: EntityAllocator,
        events: Events,
    ) -> Self {
        Self {
            components,
            allocator,
            events,
        }
    }
}

#[cfg(test)]
#[path = "tests/world_tests.rs"]
mod tests;
