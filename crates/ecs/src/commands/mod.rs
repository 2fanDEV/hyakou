mod command_buffer;

pub use command_buffer::CommandBuffer;

use crate::EntityId;

#[derive(Debug, PartialEq)]
pub enum Command {
    World(WorldCommand),
}

#[derive(Debug, PartialEq)]
pub enum WorldCommand {
    SPAWN,
    DESPAWN(EntityId),
}
