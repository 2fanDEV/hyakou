mod command_buffer;

pub use command_buffer::CommandBuffer;

use crate::EntityId;

#[derive(Debug, PartialEq)]
pub enum Command {
    Entity(EntityCommand),
}

#[derive(Debug, PartialEq)]
pub enum EntityCommand {
    Spawn,
    Despawn(EntityId),
}
