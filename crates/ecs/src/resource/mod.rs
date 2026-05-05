use std::fmt::Debug;

use type_map::TypeMap;

/// World-owned singleton data, stored independently from per-entity components.
pub trait Resource: 'static + Debug {}

#[derive(Debug, Default)]
pub struct Resources {
    values: TypeMap,
}

impl Resources {
    pub fn new(values: TypeMap) -> Self {
        Self { values }
    }

    pub fn insert<R: Resource>(&mut self, resource: R) -> Option<R> {
        self.values.insert(resource)
    }

    pub fn contains<R: Resource>(&self) -> bool {
        self.values.contains::<R>()
    }

    pub fn get<R: Resource>(&self) -> Option<&R> {
        self.values.get::<R>()
    }

    pub fn get_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.values.get_mut::<R>()
    }

    pub fn remove<R: Resource>(&mut self) -> Option<R> {
        self.values.remove::<R>()
    }
}

#[cfg(test)]
#[path = "tests/resource_trait_test.rs"]
mod resource_trait_test;
