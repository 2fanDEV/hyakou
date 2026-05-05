use crate::{EntityId, component::Component};

#[derive(Debug)]
struct TestComponent;
#[derive(Debug)]
struct TestComponent2;
impl Component for TestComponent {}
impl Component for TestComponent2 {}

#[test]
fn test_component_same_type() {
    assert_eq!(
        std::any::TypeId::of::<TestComponent>(),
        std::any::TypeId::of::<TestComponent>()
    );
}

#[test]
fn test_component_different_type() {
    assert_ne!(
        std::any::TypeId::of::<TestComponent>(),
        std::any::TypeId::of::<TestComponent2>()
    );
}

#[test]
fn test_component_storages_are_keyed_by_component_type() {
    let mut components = crate::component::Components::default();
    let entity = EntityId::new_uuid(0, 0);

    components.insert(entity.clone(), TestComponent);
    components.insert(entity, TestComponent2);

    assert!(components.contains_storage::<TestComponent>());
    assert!(components.contains_storage::<TestComponent2>());
    assert_eq!(components.storage_len::<TestComponent>(), 1);
    assert_eq!(components.storage_len::<TestComponent2>(), 1);
}
