use std::ops::RangeBounds;

use crate::commands::Command;

#[derive(Debug, Default)]
pub struct CommandBuffer {
    commands: Vec<Command>,
}

impl CommandBuffer {
    pub fn new(commands: Vec<Command>) -> Self {
        Self { commands }
    }

    pub fn push(&mut self, command: Command) {
        self.commands.push(command);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Command> {
        self.commands.iter()
    }

    pub fn drain(&mut self, range: impl RangeBounds<usize>) -> impl Iterator<Item = Command> {
        self.commands.drain(range)
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

#[cfg(test)]
#[path = "tests/command_buffer_tests.rs"]
mod command_buffer_tests;
