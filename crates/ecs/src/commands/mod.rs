mod command_buffer;

use std::fmt::{self, Debug};

pub use command_buffer::CommandBuffer;

use crate::EntityId;

#[derive(Debug, Clone)]
pub enum ComponentCommand<C> {
    Insert { entity: EntityId, component: C },
    Remove { entity: EntityId },
}

pub struct ExecutedComponentCommand<C> {
    pub command: ComponentCommand<C>,
    pub timestamp: u128,
}

impl<C: Debug> Debug for ExecutedComponentCommand<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecutedComponentCommand")
            .field("command", &self.command)
            .field("timestamp", &self.timestamp)
            .finish()
    }
}
