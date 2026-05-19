use std::fmt::Debug;

use rayon::iter::{ParallelBridge, ParallelIterator};
use shared::Shared;

use crate::component::recorder::ComponentRecorder;
use crate::component::storage::ComponentStorage;
use crate::{EntityAllocator, EntityId, storage::TypeStorage};

mod recorder;
mod storage;

pub trait Component: 'static + Send + Debug + Clone {}

#[derive(Debug, Default)]
pub struct Components {
    storages: TypeStorage,
    allocator: Shared<EntityAllocator>,
}

impl Components {
    pub fn new(allocator: Shared<EntityAllocator>) -> Self {
        Self {
            storages: TypeStorage::default(),
            allocator,
        }
    }

    pub(super) fn allocator(&self) -> &Shared<EntityAllocator> {
        &self.allocator
    }

    pub fn record(&mut self) -> ComponentRecorder<'_> {
        ComponentRecorder { components: self }
    }

    /// Applies deferred component commands.
    ///
    /// Commands are FIFO within each component type. Different component types may be applied in
    /// any order, so systems must not depend on global ordering between component types. Commands
    /// targeting dead or stale entities are skipped and logged as warnings.
    pub fn apply_commands(&mut self) {
        let allocator = self.allocator.clone();
        let storages = self.storages.filter(|stor| stor.has_outstanding_commands());
        storages
            .par_bridge()
            .for_each(|storage| storage.apply_outstanding_commands(&allocator));
    }

    pub(crate) fn insert<C: Component>(&mut self, entity: &EntityId, component: C) {
        self.storage_mut::<C>().insert(entity, component);
    }

    pub(crate) fn get<C: Component>(&self, entity: &EntityId) -> Option<&C> {
        self.storage::<C>()?.get(entity)
    }

    pub(crate) fn get_mut<C: Component>(&mut self, entity: &EntityId) -> Option<&mut C> {
        self.storage_mut::<C>().get_mut(entity)
    }

    pub fn contains_storage<C: Component>(&self) -> bool {
        self.storages.contains::<ComponentStorage<C>>()
    }

    pub fn storage_len<C: Component>(&self) -> usize {
        self.storage::<C>().map_or(0, |s| s.len())
    }

    pub fn remove_entity(&mut self, entity: &EntityId) -> usize {
        let mut removed = 0;
        for storage in self.storages.iter_mut() {
            if storage.remove_any_key(entity) {
                removed += 1;
            }
        }
        removed
    }

    pub fn remove_component<C: Component>(&mut self) -> Option<ComponentStorage<C>> {
        self.storages.remove::<ComponentStorage<C>>()
    }

    fn storage<C: Component>(&self) -> Option<&ComponentStorage<C>> {
        self.storages.get::<ComponentStorage<C>>()
    }

    fn storage_mut<C: Component>(&mut self) -> &mut ComponentStorage<C> {
        self.storages
            .get_or_insert_with::<ComponentStorage<C>, _>(|| ComponentStorage::<C>::new())
    }
}

#[cfg(test)]
#[path = "tests/component_trait_test.rs"]
mod component_trait_test;
