use crate::{Resources, resource::Resource};

#[derive(Debug, PartialEq)]
struct TestResource(u32);

#[derive(Debug, PartialEq)]
struct OtherResource(&'static str);

impl Resource for TestResource {}
impl Resource for OtherResource {}

#[test]
fn test_resource_same_type() {
    assert_eq!(
        std::any::TypeId::of::<TestResource>(),
        std::any::TypeId::of::<TestResource>()
    );
}

#[test]
fn test_resource_different_type() {
    assert_ne!(
        std::any::TypeId::of::<TestResource>(),
        std::any::TypeId::of::<OtherResource>()
    );
}

#[test]
fn test_resources_store_values_by_type() {
    let mut resources = Resources::default();

    resources.insert(TestResource(7));
    resources.insert(OtherResource("world"));

    assert_eq!(resources.get::<TestResource>(), Some(&TestResource(7)));
    assert_eq!(
        resources.get::<OtherResource>(),
        Some(&OtherResource("world"))
    );
}
