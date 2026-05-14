use crate::commands::EntityCommand;
use crate::component::Components;
use crate::CommandBuffer;
use crate::{Component, EntityAllocator, EntityId, Event, Events, Resources, resource::Resource};

#[derive(Debug, Default)]
pub struct World {
    components: Components,
    allocator: EntityAllocator,
    resources: Resources,
    events: Events,
}

impl World {
    pub fn new(
        components: Components,
        allocator: EntityAllocator,
        resources: Resources,
        events: Events,
    ) -> Self {
        Self {
            components,
            allocator,
            resources,
            events,
        }
    }

    pub fn apply_command_buffer(&mut self, buffer: &mut CommandBuffer<EntityCommand>) {
        for command in buffer.drain(..) {
            match command {
                entity_command => match entity_command {
                    EntityCommand::Spawn => {
                        self.spawn();
                    }
                    EntityCommand::Despawn(id) => {
                        self.despawn(&id);
                    }
                },
            }
        }
        self.cascading_apply();
    }

    pub fn cascading_apply(&self) {
       self.components.
    }

    pub fn spawn(&mut self) -> EntityId {
        self.allocator.spawn()
    }

    /// Despawns an entity and removes all its attached components.
    ///
    /// # Contract
    /// - If the entity is dead, stale, or unknown, returns `false` and does not mutate components.
    /// - On success, marks the entity dead first, then removes all components owned by exactly this `EntityId`.
    /// - Cleanup does not touch components belonging to other entities.
    pub fn despawn(&mut self, entity: &EntityId) -> bool {
        if !self.allocator.despawn(entity) {
            return false;
        }
        self.components.remove_entity(entity);
        true
    }

    pub fn is_alive(&self, entity: &EntityId) -> bool {
        self.allocator.is_alive(entity)
    }

    pub fn entity_count(&self) -> usize {
        self.allocator.alive_count()
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
        self.components.remove::<C>(entity);
    }
}

#[cfg(test)]
#[path = "tests/world_tests.rs"]
mod tests;
