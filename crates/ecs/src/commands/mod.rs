#[derive(Debug, Default)]
pub struct CommandBuffer<T> {
    commands: Vec<T>,
}

impl<T> CommandBuffer<T> {
    pub fn new() -> Self {
        Self { commands: vec![] }
    }

    pub fn push(&mut self, command: T) {
        self.commands.push(command);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.commands.iter()
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
