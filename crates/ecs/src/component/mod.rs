use std::{collections::HashMap, fmt::Debug};

use log::error;
use type_map::TypeMap;

use crate::EntityId;

pub trait Component: 'static + Debug + Clone {}

#[derive(Debug, Default)]
pub struct Components {
    storages: TypeMap,
}

impl Components {
    pub fn new(storages: TypeMap) -> Self {
        Self { storages }
    }

    pub(crate) fn insert<C: Component>(&mut self, entity: EntityId, component: C) -> Option<C> {
        match self.get::<C>(&entity) {
            Some(c) => {
                error!(
                    "entity {:?} already has a component of type {}",
                    entity,
                    std::any::type_name::<C>()
                );
                return Some(c.clone());
            }
            None => self.storage_mut::<C>().insert(entity, component),
        }
    }

    pub(crate) fn get<C: Component>(&self, entity: &EntityId) -> Option<&C> {
        if let Some(storage) = self.storage::<C>() {
            storage.get(entity)
        } else {
            None
        }
    }

    pub(crate) fn get_mut<C: Component>(&mut self, entity: &EntityId) -> Option<&mut C> {
        self.storage_mut::<C>().get_mut(entity)
    }

    pub(crate) fn remove<C: Component>(&mut self, entity: &EntityId) -> Option<C> {
        self.storage_mut::<C>().remove(entity)
    }

    pub(crate) fn contains<C: Component>(&self, entity: &EntityId) -> bool {
        self.storage::<C>().map_or(false, |s| s.contains(entity))
    }

    pub fn contains_storage<C: Component>(&self) -> bool {
        self.storages.contains::<ComponentStorage<C>>()
    }

    pub fn storage_len<C: Component>(&self) -> usize {
        self.storages
            .get::<ComponentStorage<C>>()
            .map_or(0, ComponentStorage::len)
    }

    fn storage<C: Component>(&self) -> Option<&ComponentStorage<C>> {
        let storage = self.storages.get::<ComponentStorage<C>>();
        match storage {
            Some(storage) => Some(storage),
            None => {
                error!(
                    "ComponentStorage for type {} not populated",
                    std::any::type_name::<C>()
                );
                None
            }
        }
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

    fn get(&self, entity: &EntityId) -> Option<&C> {
        self.values.get(entity)
    }

    fn get_mut(&mut self, entity: &EntityId) -> Option<&mut C> {
        self.values.get_mut(entity)
    }

    fn remove(&mut self, entity: &EntityId) -> Option<C> {
        self.values.remove(entity)
    }

    fn contains(&self, entity: &EntityId) -> bool {
        self.values.contains_key(entity)
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

#[cfg(test)]
#[path = "tests/component_trait_test.rs"]
mod component_trait_test;
