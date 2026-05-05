use crate::component::Component;

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
