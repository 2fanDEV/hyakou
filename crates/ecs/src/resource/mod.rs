use std::any::Any;
use std::fmt::Debug;

use crate::{Storage, TypeStorage};

/// World-owned singleton data, stored independently from per-entity components.
pub trait Resource: 'static + Send + Debug {}

#[derive(Debug)]
struct ResourceValue<R> {
    value: R,
}

impl<R: Resource> Storage for ResourceValue<R> {
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

#[derive(Debug, Default)]
pub struct Resources {
    values: TypeStorage,
}

impl Resources {
    pub fn new(values: TypeStorage) -> Self {
        Self { values }
    }

    pub fn insert<R: Resource>(&mut self, resource: R) -> Option<R> {
        self.values
            .insert(ResourceValue { value: resource })
            .map(|stored| stored.value)
    }

    pub fn contains<R: Resource>(&self) -> bool {
        self.values.contains::<ResourceValue<R>>()
    }

    pub fn get<R: Resource>(&self) -> Option<&R> {
        self.values
            .get::<ResourceValue<R>>()
            .map(|stored| &stored.value)
    }

    pub fn get_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.values
            .get_mut::<ResourceValue<R>>()
            .map(|stored| &mut stored.value)
    }

    pub fn remove<R: Resource>(&mut self) -> Option<R> {
        self.values
            .remove::<ResourceValue<R>>()
            .map(|stored| stored.value)
    }
}

#[cfg(test)]
#[path = "tests/resource_trait_test.rs"]
mod resource_trait_test;
