mod events;

pub trait Event: 'static {}

#[cfg(test)]
#[path = "tests/event_trait_test.rs"]
mod event_trait_test;
