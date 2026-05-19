use anyhow::{Result, anyhow};
use log::debug;
use shared::{Shared, SharedAccess};

use crate::CommandBuffer;
use crate::component::Components;
use crate::world::commands::EntityCommand;
use crate::world::recorder::WorldRecorder;
use crate::{Component, EntityAllocator, EntityId, Event, Events, Resources, resource::Resource};

pub mod commands;
pub mod recorder;

#[derive(Debug)]
pub struct World {
    components: Components,
    allocator: Shared<EntityAllocator>,
    command_buffer: CommandBuffer<EntityCommand>,
    resources: Resources,
    events: Events,
}

impl Default for World {
    fn default() -> Self {
        let allocator: Shared<EntityAllocator> = Default::default();
        Self {
            components: Components::new(allocator.clone()),
            allocator,
            command_buffer: CommandBuffer::new(Vec::new()),
            resources: Default::default(),
            events: Default::default(),
        }
    }
}

impl World {
    pub fn new(allocator: Shared<EntityAllocator>, resources: Resources, events: Events) -> Self {
        let components = Components::new(allocator.clone());
        Self {
            components,
            allocator,
            command_buffer: CommandBuffer::new(Vec::new()),
            resources,
            events,
        }
    }

    pub fn recorder(&mut self) -> WorldRecorder<'_> {
        WorldRecorder::new(self)
    }

    pub fn apply_command_buffer(&mut self) {
        let cmd = std::mem::take(&mut self.command_buffer);
        for command in cmd.iter() {
            self.apply_command(command);
        }
        self.cascading_apply();
    }

    pub fn buffer(&self) -> &CommandBuffer<EntityCommand> {
        &self.command_buffer
    }

    pub fn cascading_apply(&mut self) {
        self.components.apply_commands();
    }

    pub fn apply_command(&mut self, command: &EntityCommand) {
        match command {
            EntityCommand::Spawn => {
                self.spawn().unwrap();
            }
            EntityCommand::Despawn(id) => {
                self.despawn(id);
            }
        }
    }

    pub fn spawn(&mut self) -> Result<EntityId> {
        self.allocator
            .try_write_shared(|alloc| alloc.spawn())
            .map_err(|e| anyhow!(e))
    }

    pub fn despawn(&mut self, entity: &EntityId) -> bool {
        match self
            .allocator
            .try_write_shared(|alloc| alloc.despawn(entity))
        {
            Ok(res) => {
                self.components.remove_entity(entity);
                res
            }
            Err(e) => {
                debug!("Failed to despawn entity: {}", e);
                false
            }
        }
    }

    pub fn is_alive(&self, entity: &EntityId) -> bool {
        self.allocator
            .try_read_shared(|alloc| alloc.is_alive(entity))
            .unwrap_or(false)
    }

    pub fn entity_count(&self) -> usize {
        self.allocator
            .try_read_shared(|alloc| alloc.alive_count())
            .unwrap_or(0)
    }

    pub fn write_event<E: Event>(&mut self, event: E) {
        self.events.write(event);
    }

    pub fn read_events<E: Event>(&self) -> &[E] {
        self.events.read::<E>()
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> Option<R> {
        self.resources.insert(resource)
    }

    pub fn resource<R: Resource>(&self) -> Option<&R> {
        self.resources.get::<R>()
    }

    pub fn resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.resources.get_mut::<R>()
    }

    pub fn remove_resource<R: Resource>(&mut self) -> Option<R> {
        self.resources.remove::<R>()
    }

    pub fn insert_component<C: Component>(&mut self, entity: &mut EntityId, component: C) {
        let is_alive = self.is_alive(entity);
        if !is_alive {
            return;
        }
        self.components.insert(entity, component);
    }

    pub fn get_component<C: Component>(&self, entity: &EntityId) -> Option<&C> {
        self.components.get::<C>(entity)
    }

    pub fn get_component_mut<C: Component>(&mut self, entity: &EntityId) -> Option<&mut C> {
        self.components.get_mut::<C>(entity)
    }

    pub fn remove_component<C: Component>(&mut self, entity: &mut EntityId) {
        self.components.remove_entity(entity);
    }
}

#[cfg(test)]
#[path = "tests/world_tests.rs"]
mod tests;
