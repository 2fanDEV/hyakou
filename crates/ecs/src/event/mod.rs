mod events;
use std::any::Any;
use std::fmt::Debug;

pub use events::Events;

use crate::Storage;

pub trait Event: 'static + Debug {}

#[derive(Debug)]
pub(super) struct EventQueue<E: Event> {
    pub(super) events: Vec<E>,
}

impl<E: Event> Storage for EventQueue<E> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

#[cfg(test)]
#[path = "tests/event_trait_test.rs"]
mod event_trait_test;
