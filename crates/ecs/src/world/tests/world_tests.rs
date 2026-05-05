use crate::{
    Component, Components, EntityAllocator, Event, Events, Resource, Resources, world::World,
};

#[derive(Debug, PartialEq)]
struct TestComponent;

#[derive(Debug, PartialEq)]
struct TestEvent(u32);

#[derive(Debug, PartialEq)]
struct TestResource(u32);

impl Component for TestComponent {}
impl Event for TestEvent {}
impl Resource for TestResource {}

#[test]
pub fn empty_world_test() {
    let world = World::default();

    assert_eq!(world.components.storage_len::<TestComponent>(), 0);
}

#[test]
fn test_world_spawn_and_despawn() {
    let mut world = World::default();
    let entity = world.spawn();

    assert!(world.is_alive(&entity));
    assert!(world.despawn(&entity));
    assert!(!world.is_alive(&entity));
}

#[test]
fn test_world_double_despawn_fails_safely() {
    let mut world = World::default();
    let entity = world.spawn();

    assert!(world.despawn(&entity));
    assert!(!world.despawn(&entity));
}

#[test]
fn test_world_stale_despawn_fails_safely() {
    let mut world = World::default();
    let stale_entity = world.spawn();

    assert!(world.despawn(&stale_entity));

    let new_entity = world.spawn();

    assert!(!world.despawn(&stale_entity));
    assert!(world.is_alive(&new_entity));
}

#[test]
fn test_world_events_are_written_and_read_by_type() {
    let mut world = World::default();

    world.write_event(TestEvent(1));
    world.write_event(TestEvent(2));

    assert_eq!(
        world.read_events::<TestEvent>(),
        &[TestEvent(1), TestEvent(2)]
    );
}

#[test]
fn test_world_resources_are_isolated() {
    let mut first_world = World::default();
    let mut second_world = World::default();

    first_world.insert_resource(TestResource(1));
    second_world.insert_resource(TestResource(2));

    first_world.resource_mut::<TestResource>().unwrap().0 = 3;

    assert_eq!(
        first_world.resource::<TestResource>(),
        Some(&TestResource(3))
    );
    assert_eq!(
        second_world.resource::<TestResource>(),
        Some(&TestResource(2))
    );

    assert_eq!(
        first_world.remove_resource::<TestResource>(),
        Some(TestResource(3))
    );
    assert_eq!(first_world.resource::<TestResource>(), None);
    assert_eq!(
        second_world.resource::<TestResource>(),
        Some(&TestResource(2))
    );
}

#[test]
fn test_world_component_storage_is_world_local() {
    let mut first_world = World::default();
    let second_world = World::default();
    let entity = first_world.spawn();

    first_world.insert_component(entity, TestComponent);

    assert_eq!(first_world.components.storage_len::<TestComponent>(), 1);
    assert_eq!(second_world.components.storage_len::<TestComponent>(), 0);
}

#[test]
fn test_world_new_uses_injected_state() {
    let mut allocator = EntityAllocator::default();
    let entity = allocator.spawn();
    let world = World::new(
        Components::default(),
        allocator,
        Resources::default(),
        Events::default(),
    );

    assert!(world.is_alive(&entity));
}
