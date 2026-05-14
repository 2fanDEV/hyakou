use std::any::Any;
use std::collections::HashMap;
use std::default;
use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};

use rayon::iter::{ParallelBridge, ParallelIterator};
use shared::Shared;

use crate::commands::{ComponentCommand, ExecutedComponentCommand};
use crate::storage::{KeyedStorage, Storage, TypeStorage};
use crate::{CommandBuffer, EntityAllocator, EntityId};

pub trait Component: 'static + Send + Debug + Clone {}

#[derive(Debug)]
struct ComponentStorage<C: Component> {
    values: HashMap<EntityId, C>,
    outstanding_commands: CommandBuffer<ComponentCommand<C>>,
    executed_commands: Vec<ExecutedComponentCommand<C>>,
}

impl<C: Component> ComponentStorage<C> {
    fn insert_command(&mut self, command: ComponentCommand<C>) {
        self.outstanding_commands.push(command);
    }

    fn insert(&mut self, entity: EntityId, component: C) {
        self.values.insert(entity, component);
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

    pub fn has_outstanding_commands(&self) -> bool {
        !self.outstanding_commands.is_empty()
    }

    pub fn apply_outstanding_commands(&mut self) {
        let drainage = self.outstanding_commands.drain(..).collect::<Vec<_>>();
        for command in drainage {
            let cmd = command.clone();
            match command {
                ComponentCommand::Insert { entity, component } => {
                    self.insert(entity, component);
                }
                ComponentCommand::Remove { entity } => {
                    self.remove(&entity);
                }
            }
            self.executed_commands.push(ExecutedComponentCommand {
                command: cmd,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis(),
            });
        }
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
    allocator: Shared<EntityAllocator>,
}

impl Components {
    pub fn new(allocator: Shared<EntityAllocator>) -> Self {
        Self {
            storages: TypeStorage::default(),
            allocator,
        }
    }

    pub fn apply_commands(&mut self) {
        let storages = self.storages.filter(|stor| stor.has_outstanding_commands());
        storages
            .par_bridge()
            .for_each(|storage| storage.apply_outstanding_commands());
    }

    pub fn insert<C: Component>(&mut self, entity: &mut EntityId, component: C) {
        if self.allocator.borrow().is_alive(entity) {
            self.storage_mut::<C>()
                .insert_command(ComponentCommand::Insert {
                    entity: entity.clone(),
                    component,
                });
        }
    }

    pub(crate) fn get<C: Component>(&self, entity: &EntityId) -> Option<&C> {
        self.storage::<C>()?.get(entity)
    }

    pub(crate) fn get_mut<C: Component>(&mut self, entity: &EntityId) -> Option<&mut C> {
        self.storage_mut::<C>().get_mut(entity)
    }

    pub fn remove<C: Component>(&mut self, entity: &EntityId) -> Option<C> {
        self.storage_mut::<C>()
            .insert_command(ComponentCommand::Remove {
                entity: entity.clone(),
            });
        None
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

    fn storage<C: Component>(&self) -> Option<&ComponentStorage<C>> {
        self.storages.get::<ComponentStorage<C>>()
    }

    fn storage_mut<C: Component>(&mut self) -> &mut ComponentStorage<C> {
        self.storages
            .get_or_insert_with::<ComponentStorage<C>, _>(|| ComponentStorage::<C> {
                values: HashMap::default(),
                outstanding_commands: CommandBuffer::<ComponentCommand<C>>::empty(),
                executed_commands: Vec::default(),
            })
    }
}

#[cfg(test)]
#[path = "tests/component_trait_test.rs"]
mod component_trait_test;
