use crate::{Component, EntityAllocator, Events};

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
