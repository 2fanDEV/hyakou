use crate::{Events, event::Event};

#[derive(Debug, PartialEq)]
struct TestEvent(u32);

#[derive(Debug, PartialEq)]
struct TestEvent2(&'static str);

impl Event for TestEvent {}
impl Event for TestEvent2 {}

#[test]
fn test_event_same_type() {
    assert_eq!(
        std::any::TypeId::of::<TestEvent>(),
        std::any::TypeId::of::<TestEvent>()
    );
}

#[test]
fn test_event_different_type() {
    assert_ne!(
        std::any::TypeId::of::<TestEvent>(),
        std::any::TypeId::of::<TestEvent2>()
    );
}

#[test]
fn test_events_write_and_read_by_type_in_order() {
    let mut events = Events::default();

    events.write(TestEvent(1));
    events.write(TestEvent(2));
    events.write(TestEvent(3));

    assert_eq!(
        events.read::<TestEvent>(),
        &[TestEvent(1), TestEvent(2), TestEvent(3)]
    );
}
