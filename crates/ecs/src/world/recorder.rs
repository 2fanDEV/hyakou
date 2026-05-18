use crate::{Entity, World};

pub struct WorldRecorder<'a> {
    world: &'a mut World,
}

impl<'a> WorldRecorder<'a> {
    pub fn new(world: &'a mut World) -> Self {
        Self { world }
    }

    pub fn spawn(&mut self) -> Entity {
        let entity = self.world.command_buffer.push(cmd);
        entity
    }
}
