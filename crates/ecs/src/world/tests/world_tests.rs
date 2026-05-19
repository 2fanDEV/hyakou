use crate::{
    CommandBuffer, Component, EntityId, Event, Resource,
    world::{World, commands::EntityCommand, recorder},
};

#[derive(Debug, PartialEq, Clone)]
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
    let mut entity = world.spawn().unwrap();

    assert!(world.is_alive(&entity));
    assert!(world.despawn(&mut entity));
    assert!(!world.is_alive(&entity));
}

#[test]
fn test_world_double_despawn_fails_safely() {
    let mut world = World::default();
    let mut entity = world.spawn().unwrap();
    assert!(world.despawn(&mut entity));
    assert!(!world.despawn(&mut entity));
}

#[test]
fn test_world_stale_despawn_fails_safely() {
    let mut world = World::default();
    let mut stale_entity = world.spawn().unwrap();
    assert!(world.despawn(&mut stale_entity));
    let new_entity = world.spawn().unwrap();
    assert!(!world.despawn(&mut stale_entity));
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
    let mut entity = first_world.spawn().unwrap();

    first_world.insert_component(&mut entity, TestComponent);

    assert_eq!(first_world.components.storage_len::<TestComponent>(), 1);
    assert_eq!(second_world.components.storage_len::<TestComponent>(), 0);
}

#[test]
fn test_deferred_component_insert_applies_for_alive_entity() {
    let mut world = World::default();
    let mut entity = world.spawn().unwrap();

    world.components.record().insert(&mut entity, TestComponent);
    world.cascading_apply();

    assert_eq!(
        world.get_component::<TestComponent>(&entity),
        Some(&TestComponent)
    );
}

#[test]
fn test_deferred_component_remove_applies_for_alive_entity() {
    let mut world = World::default();
    let mut entity = world.spawn().unwrap();
    world.insert_component(&mut entity, TestComponent);

    world
        .components
        .record()
        .remove::<TestComponent>(&mut entity);
    world.cascading_apply();

    assert_eq!(world.get_component::<TestComponent>(&entity), None);
}

#[test]
fn test_deferred_component_insert_is_not_recorded_for_dead_entity() {
    let mut world = World::default();
    let mut entity = world.spawn().unwrap();
    assert!(world.despawn(&entity));

    world.components.record().insert(&mut entity, TestComponent);
    world.cascading_apply();

    assert_eq!(world.get_component::<TestComponent>(&entity), None);
}

#[test]
fn test_deferred_component_insert_skips_entity_dead_before_apply() {
    let mut world = World::default();
    let mut entity = world.spawn().unwrap();

    world.components.record().insert(&mut entity, TestComponent);
    assert!(world.despawn(&entity));
    world.cascading_apply();

    assert_eq!(world.get_component::<TestComponent>(&entity), None);
}

#[test]
fn test_world_new_uses_injected_state() {
    let mut world = World::default();
    let entity = world.spawn().unwrap();
    assert!(world.is_alive(&entity));
}

#[test]
fn test_despawn_removes_attached_components() {
    let mut world = World::default();
    let mut entity = world.spawn().unwrap();
    world.insert_component(&mut entity, TestComponent);
    assert!(world.despawn(&entity));
    assert!(!world.is_alive(&entity));
    assert_eq!(world.get_component::<TestComponent>(&entity), None);
}

#[test]
fn test_despawn_does_not_affect_other_entities() {
    let mut world = World::default();
    let mut entity1 = world.spawn().unwrap();
    let mut entity2 = world.spawn().unwrap();
    world.insert_component(&mut entity1, TestComponent);
    world.insert_component(&mut entity2, TestComponent);
    assert!(world.despawn(&entity1));
    println!("{:?}", entity1);
    println!("{:?}", entity2);
    assert!(world.is_alive(&entity2));
    assert_eq!(
        world.get_component::<TestComponent>(&entity2),
        Some(&TestComponent)
    );
}

#[test]
fn test_despawn_with_no_components() {
    let mut world = World::default();
    let entity = world.spawn().unwrap();
    assert!(world.despawn(&entity));
    assert!(!world.is_alive(&entity));
}

#[test]
fn test_double_despawn_does_not_cleanup_again() {
    let mut world = World::default();
    let entity = world.spawn().unwrap();
    assert!(world.despawn(&entity));
    assert!(!world.despawn(&entity));
    assert_eq!(world.get_component::<TestComponent>(&entity), None);
}

#[test]
fn test_stale_despawn_does_not_remove_new_entity_components() {
    let mut world = World::default();
    let mut stale_entity = world.spawn().unwrap();
    world.insert_component(&mut stale_entity, TestComponent);
    assert!(world.despawn(&stale_entity));
    let mut new_entity = world.spawn().unwrap();
    world.insert_component(&mut new_entity, TestComponent);
    assert!(!world.despawn(&stale_entity));
    assert!(world.is_alive(&new_entity));
    assert_eq!(
        world.get_component::<TestComponent>(&new_entity),
        Some(&TestComponent)
    );
}

#[test]
fn test_entity_spawn_via_command_buffer() {
    let mut world = World::default();
    let mut recorder = world.recorder();
    recorder.spawn(EntityId::default());
    recorder.spawn(EntityId::default());
    world.apply_command_buffer();
    assert_eq!(world.entity_count(), 2);
    assert!(world.buffer().is_empty());
}

#[test]
fn test_entity_despawn_via_command_buffer() {
    let mut world = World::default();
    let entity = EntityId::default();
    world.recorder().spawn(entity.clone());
    world.apply_command_buffer();
    assert!(world.is_alive(&entity));
    world.recorder().despawn(entity.clone());
    world.apply_command_buffer();
    assert!(!world.is_alive(&entity));
    assert_eq!(world.entity_count(), 0);
}

#[test]
fn test_entity_spawn_then_despawn_ordering_via_buffer() {
    let mut world = World::default();
    let entity = world.spawn().unwrap();
    assert_eq!(world.entity_count(), 1);
    let mut recorder = world.recorder();
    recorder.spawn(entity.clone());
    recorder.despawn(entity.clone());
    world.apply_command_buffer();
    assert!(!world.is_alive(&entity));
    assert_eq!(world.entity_count(), 1);
    assert!(world.buffer().is_empty())
}

#[test]
fn test_entity_stale_despawn_via_buffer() {
    let mut world = World::default();
    let id = EntityId::default();
    world.recorder().spawn(id.clone());
    world.recorder().despawn(id.clone());
    world.apply_command_buffer();
    assert!(!world.is_alive(&id));
    assert_eq!(world.entity_count(), 0);
}

#[test]
fn test_entity_empty_command_buffer_does_nothing() {
    let mut world = World::default();
    world.spawn().unwrap();
    world.apply_command_buffer();
    assert_eq!(world.entity_count(), 1);
}
