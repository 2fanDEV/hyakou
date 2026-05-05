use type_map::TypeMap;

use crate::Event;

#[derive(Debug, Default)]
pub struct Events {
    queues: TypeMap,
}

impl Events {
    pub fn new(queues: TypeMap) -> Self {
        Self { queues }
    }

    pub fn write<E: Event>(&mut self, event: E) {
        self.queues
            .entry::<Vec<E>>()
            .or_insert_with(Vec::new)
            .push(event);
    }

    pub fn len<E: Event>(&self) -> usize {
        self.read::<E>().len()
    }

    pub fn is_empty<E: Event>(&self) -> bool {
        self.read::<E>().is_empty()
    }

    pub fn read<E: Event>(&self) -> &[E] {
        self.queues.get::<Vec<E>>().map_or(&[], Vec::as_slice)
    }
}
