use crate::Component;

#[derive(Debug)]
pub struct World {
    components: Vec<Box<dyn Component>>,
}

impl World {
    pub fn new() -> Self {
        Self { components: vec![] }
    }
}

#[cfg(test)]
#[path = "tests/world_tests.rs"]
mod tests;
