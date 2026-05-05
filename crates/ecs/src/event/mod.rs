mod events;
use std::fmt::Debug;

pub use events::Events;

pub trait Event: 'static + Debug {}

#[cfg(test)]
#[path = "tests/event_trait_test.rs"]
mod event_trait_test;
