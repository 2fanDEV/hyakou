use std::{collections::HashMap, fmt::Debug};

use type_map::TypeMap;

use crate::EntityId;

pub trait Component: 'static + Debug {}

#[derive(Debug, Default)]
pub struct Components {
    storages: TypeMap,
}

impl Components {
    pub fn new(storages: TypeMap) -> Self {
        Self { storages }
    }

    pub(crate) fn insert<C: Component>(&mut self, entity: EntityId, component: C) -> Option<C> {
        self.storage_mut::<C>().insert(entity, component)
    }

    pub fn contains_storage<C: Component>(&self) -> bool {
        self.storages.contains::<ComponentStorage<C>>()
    }

    pub fn storage_len<C: Component>(&self) -> usize {
        self.storages
            .get::<ComponentStorage<C>>()
            .map_or(0, ComponentStorage::len)
    }

    fn storage_mut<C: Component>(&mut self) -> &mut ComponentStorage<C> {
        self.storages
            .entry::<ComponentStorage<C>>()
            .or_insert_with(|| ComponentStorage {
                values: HashMap::default(),
            })
    }
}

#[derive(Debug)]
struct ComponentStorage<C> {
    values: HashMap<EntityId, C>,
}

impl<C> ComponentStorage<C> {
    fn insert(&mut self, entity: EntityId, component: C) -> Option<C> {
        self.values.insert(entity, component)
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

#[cfg(test)]
#[path = "tests/component_trait_test.rs"]
mod component_trait_test;
