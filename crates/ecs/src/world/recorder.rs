use crate::{EntityId, World, world::commands::EntityCommand};

pub struct WorldRecorder<'a> {
    world: &'a mut World,
}

impl<'a> WorldRecorder<'a> {
    pub fn new(world: &'a mut World) -> Self {
        Self { world }
    }

    pub fn spawn(&mut self, entity_id: EntityId) {
        self.world
            .command_buffer
            .push(EntityCommand::Spawn(entity_id));
    }

    pub fn despawn(&mut self, entity: EntityId) {
        self.world
            .command_buffer
            .push(EntityCommand::Despawn(entity));
    }
}
