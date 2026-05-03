use std::{any::TypeId, collections::HashMap};

use crate::Event;

pub struct Events {
    pub buckets: HashMap<TypeId, Vec<Box<dyn Event>>>,
}

impl Events {
    pub fn new() -> Self {
        Self {
            buckets: HashMap::new(),
        }
    }

    pub fn push<E: Event>(&mut self, event: E) {
        self.buckets
            .entry(TypeId::of::<E>())
            .or_default()
            .push(Box::new(event));
    }

    pub fn len<E: Event>(&self) -> usize {
        self.buckets.get(&TypeId::of::<E>()).map_or(0, Vec::len)
    }

    pub fn get<E: Event>(&self) -> Option<&[Box<dyn Event>]> {
        let type_id = TypeId::of::<E>();
        self.buckets.get(&type_id).map(Vec::as_slice)
    }
}
