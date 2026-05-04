use crate::world::World;

#[test]
pub fn empty_world_test() {
    let world = World::new();

    assert!(world.components.is_empty())
}
