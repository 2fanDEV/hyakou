use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

use log::error;

use crate::EntityId;
use crate::storage::{KeyedStorage, Storage, TypeStorage};

pub trait Component: 'static + Debug + Clone {}

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

    fn len(&self) -> usize {
        self.values.len()
    }
}

impl<C: Component> Storage for ComponentStorage<C> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn remove_any_key(&mut self, key: &dyn Any) -> bool {
        key.downcast_ref::<EntityId>()
            .is_some_and(|entity| self.remove_key(entity))
    }
}

impl<C: Component> KeyedStorage<EntityId> for ComponentStorage<C> {
    fn remove_key(&mut self, key: &EntityId) -> bool {
        self.values.remove(key).is_some()
    }
}

#[derive(Debug, Default)]
pub struct Components {
    storages: TypeStorage,
}

impl Components {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn insert<C: Component>(
        &mut self,
        entity: &mut EntityId,
        component: C,
    ) -> Option<C> {
        if let Some(c) = self.get::<C>(entity) {
            error!(
                "entity {:?} already has a component of type {}",
                entity,
                std::any::type_name::<C>()
            );
            return Some(c.clone());
        }

        self.storage_mut::<C>().insert(entity.clone(), component)
    }

    pub(crate) fn get<C: Component>(&self, entity: &EntityId) -> Option<&C> {
        self.storage::<C>()?.get(entity)
    }

    pub(crate) fn get_mut<C: Component>(&mut self, entity: &EntityId) -> Option<&mut C> {
        self.storage_mut::<C>().get_mut(entity)
    }

    pub(crate) fn remove<C: Component>(&mut self, entity: &EntityId) -> Option<C> {
        self.storage_mut::<C>().remove(entity)
    }

    pub fn contains_storage<C: Component>(&self) -> bool {
        self.storages.contains::<ComponentStorage<C>>()
    }

    pub fn storage_len<C: Component>(&self) -> usize {
        self.storage::<C>().map_or(0, |s| s.len())
    }

    pub(crate) fn remove_entity(&mut self, entity: &EntityId) -> usize {
        let mut removed = 0;
        for storage in self.storages.iter_mut() {
            if storage.remove_any_key(entity) {
                removed += 1;
            }
        }
        removed
    }

    fn storage<C: Component>(&self) -> Option<&ComponentStorage<C>> {
        self.storages.get::<ComponentStorage<C>>()
    }

    fn storage_mut<C: Component>(&mut self) -> &mut ComponentStorage<C> {
        self.storages
            .get_or_insert_with::<ComponentStorage<C>, _>(|| ComponentStorage::<C> {
                values: HashMap::default(),
            })
    }
}

#[cfg(test)]
#[path = "tests/component_trait_test.rs"]
mod component_trait_test;
