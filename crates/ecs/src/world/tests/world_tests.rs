use crate::{EntityAllocator, Events, world::World};

#[test]
pub fn empty_world_test() {
    let world = World::new(vec![], EntityAllocator::new(), Events::new());

    assert!(world.components.is_empty())
}
