use crate::{EntityId, component::Component};

#[derive(Debug, Clone, PartialEq)]
struct TestComponent;

#[derive(Debug, Clone, PartialEq)]
struct TestComponent2;

#[derive(Debug, Clone, PartialEq)]
struct Health(i32);

#[derive(Debug, Clone, PartialEq)]
struct Mana(i32);

impl Component for TestComponent {}
impl Component for TestComponent2 {}
impl Component for Health {}
impl Component for Mana {}

#[test]
fn test_distinct_component_types_are_stored_independently() {
    let mut components = crate::component::Components::default();
    let mut entity = EntityId::new_uuid(0, 0);

    components.insert(&mut entity, TestComponent);
    components.insert(&mut entity, TestComponent2);
    components.apply_commands();
    assert!(components.get::<TestComponent>(&entity).is_some());
    assert!(components.get::<TestComponent2>(&entity).is_some());
    assert_eq!(components.storage_len::<TestComponent>(), 1);
    assert_eq!(components.storage_len::<TestComponent2>(), 1);
}

#[test]
fn test_same_component_type_reuses_one_storage() {
    let mut components = crate::component::Components::default();
    let mut entity1 = EntityId::new_uuid(0, 0);
    let mut entity2 = EntityId::new_uuid(1, 0);

    components.insert(&mut entity1, TestComponent);
    components.insert(&mut entity2, TestComponent);
    components.apply_commands();
    assert!(components.get::<TestComponent>(&entity1).is_some());
    assert!(components.get::<TestComponent>(&entity2).is_some());
    assert_eq!(components.storage_len::<TestComponent>(), 2);
}

#[test]
fn test_mutable_access_updates_only_requested_component_type() {
    let mut components = crate::component::Components::default();
    let mut entity = EntityId::new_uuid(0, 0);

    components.insert(&mut entity, Health(100));
    components.insert(&mut entity, Mana(50));
    components.apply_commands();
    {
        let health = components.get_mut::<Health>(&entity).unwrap();
        health.0 = 200;
    }

    assert_eq!(components.get::<Health>(&entity).unwrap().0, 200);
    assert_eq!(components.get::<Mana>(&entity).unwrap().0, 50);
}

#[test]
fn test_remove_one_component_type_leaves_other_component_type_intact() {
    let mut components = crate::component::Components::default();
    let mut entity = EntityId::new_uuid(0, 0);

    components.insert(&mut entity, TestComponent);
    components.insert(&mut entity, TestComponent2);

    components.remove_component::<TestComponent>();
    assert!(components.get::<TestComponent>(&entity).is_none());
    assert!(components.get::<TestComponent2>(&entity).is_some());
    assert_eq!(components.storage_len::<TestComponent>(), 0);
    assert_eq!(components.storage_len::<TestComponent2>(), 1);
}

#[test]
fn test_components_remove_entity_removes_all_types() {
    let mut components = crate::component::Components::default();
    let mut entity = EntityId::new_uuid(0, 0);

    components.insert(&mut entity, TestComponent);
    components.insert(&mut entity, TestComponent2);
    components.apply_commands();
    let removed = components.remove_entity(&entity);
    assert_eq!(removed, 2);
    assert!(components.get::<TestComponent>(&entity).is_none());
    assert!(components.get::<TestComponent2>(&entity).is_none());
}

#[test]
fn test_components_remove_entity_returns_zero_for_unknown_entity() {
    let mut components = crate::component::Components::default();
    let entity = EntityId::new_uuid(0, 0);

    let removed = components.remove_entity(&entity);
    assert_eq!(removed, 0);
}

#[test]
fn test_components_remove_entity_does_not_affect_other_entities() {
    let mut components = crate::component::Components::default();
    let mut entity1 = EntityId::new_uuid(0, 0);
    let mut entity2 = EntityId::new_uuid(1, 0);

    components.insert(&mut entity1, TestComponent);
    components.insert(&mut entity2, TestComponent);
    components.apply_commands();
    let removed = components.remove_entity(&entity1);
    assert_eq!(removed, 1);
    assert!(components.get::<TestComponent>(&entity1).is_none());
    assert!(components.get::<TestComponent>(&entity2).is_some());
}
