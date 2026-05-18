use crate::{Component, Components, EntityId, commands::ComponentCommand};
use shared::SharedAccess;

pub struct ComponentRecorder<'a> {
    pub components: &'a mut Components,
}

impl<'a> ComponentRecorder<'a> {
    pub fn insert<C: Component>(&mut self, entity: &mut EntityId, component: C) {
        let allocator = self.components.allocator();
        if let Ok(true) = allocator.try_read_shared(|alloc| alloc.is_alive(entity)) {
            self.components
                .storage_mut::<C>()
                .insert_command(ComponentCommand::Insert {
                    entity: entity.clone(),
                    component,
                });
        }
    }

    pub fn remove<C: Component>(&mut self, entity: &mut EntityId) {
        let allocator = self.components.allocator();
        if let Ok(true) = allocator.try_read_shared(|alloc| alloc.is_alive(entity)) {
            {
                self.components
                    .storage_mut::<C>()
                    .insert_command(ComponentCommand::Remove {
                        entity: entity.clone(),
                    });
            }
        }
    }
}
