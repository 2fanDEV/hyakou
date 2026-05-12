use std::ops::RangeBounds;

#[derive(Debug, Default)]
pub struct CommandBuffer<C> {
    commands: Vec<C>,
}

impl<C> CommandBuffer<C> {
    pub fn new(commands: Vec<C>) -> Self {
        Self { commands }
    }

    pub fn push(&mut self, C: C) {
        self.commands.push(C);
    }

    pub fn iter(&self) -> impl Iterator<Item = &C> {
        self.commands.iter()
    }

    pub fn drain(&mut self, range: impl RangeBounds<usize>) -> impl Iterator<Item = C> {
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
