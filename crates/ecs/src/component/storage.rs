use std::{
    any::Any,
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use log::warn;
use shared::{Shared, SharedAccess};

use crate::{
    CommandBuffer, Component, EntityAllocator, EntityId, KeyedStorage, Storage,
    commands::{ComponentCommand, ExecutedComponentCommand},
};

#[derive(Debug)]
pub struct ComponentStorage<C: Component> {
    values: HashMap<EntityId, C>,
    outstanding_commands: CommandBuffer<ComponentCommand<C>>,
    executed_commands: Vec<ExecutedComponentCommand<C>>,
}

impl<C: Component> ComponentStorage<C> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            outstanding_commands: CommandBuffer::<ComponentCommand<C>>::default(),
            executed_commands: Vec::new(),
        }
    }

    pub fn insert_command(&mut self, command: ComponentCommand<C>) {
        self.outstanding_commands.push(command);
    }

    pub fn insert(&mut self, entity: &EntityId, component: C) {
        self.values.insert(entity.clone(), component);
    }

    pub fn get(&self, entity: &EntityId) -> Option<&C> {
        self.values.get(entity)
    }

    pub fn get_mut(&mut self, entity: &EntityId) -> Option<&mut C> {
        self.values.get_mut(entity)
    }

    pub fn remove(&mut self, entity: &EntityId) -> Option<C> {
        self.values.remove(entity)
    }

    pub fn len(&self) -> usize {
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
            .is_some_and(|entity| self.remove(entity).is_some())
    }

    fn has_outstanding_commands(&self) -> bool {
        !self.outstanding_commands.is_empty()
    }

    fn apply_outstanding_commands(&mut self, allocator: &Shared<EntityAllocator>) {
        let drainage = self.outstanding_commands.drain(..).collect::<Vec<_>>();
        let is_alive = |entity: &EntityId| {
            allocator
                .try_read_shared(|alloc| alloc.is_alive(entity))
                .unwrap_or(false)
        };

        for command in drainage {
            let cmd = command.clone();
            match command {
                ComponentCommand::Insert { entity, component } => {
                    if is_alive(&entity) {
                        self.insert(&entity, component);
                    } else {
                        warn!("Skipped component insert for dead entity: {:?}", entity);
                    }
                }
                ComponentCommand::Remove { entity } => {
                    if is_alive(&entity) {
                        self.remove(&entity);
                    } else {
                        warn!("Skipped component remove for dead entity: {:?}", entity);
                    }
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

impl<C: Component> KeyedStorage<EntityId> for ComponentStorage<C> {
    fn remove_key(&mut self, key: &EntityId) -> bool {
        self.values.remove(key).is_some()
    }
}
