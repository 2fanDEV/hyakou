use super::EventQueue;
use crate::{Event, TypeStorage};

#[derive(Debug, Default)]
pub struct Events {
    queues: TypeStorage,
}

impl Events {
    pub fn new(queues: TypeStorage) -> Self {
        Self { queues }
    }

    pub fn write<E: Event>(&mut self, event: E) {
        self.queues
            .get_or_insert_with::<EventQueue<E>, _>(|| EventQueue { events: Vec::new() })
            .events
            .push(event);
    }

    pub fn len<E: Event>(&self) -> usize {
        self.read::<E>().len()
    }

    pub fn is_empty<E: Event>(&self) -> bool {
        self.read::<E>().is_empty()
    }

    pub fn read<E: Event>(&self) -> &[E] {
        self.queues
            .get::<EventQueue<E>>()
            .map_or(&[], |queue| queue.events.as_slice())
    }
}
