mod command_buffer;

pub use command_buffer::CommandBuffer;

use crate::EntityId;

#[derive(Debug, PartialEq)]
pub enum EntityCommand {
    Spawn,
    Despawn(EntityId),
}

#[derive(Debug, Clone)]
pub enum ComponentCommand<C> {
    Insert { entity: EntityId, component: C },
    Remove { entity: EntityId },
}

#[derive(Debug)]
pub struct ExecutedComponentCommand<C> {
    pub command: ComponentCommand<C>,
    pub timestamp: u128,
}
