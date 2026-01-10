use std::collections::HashMap;

use log::{error, trace};
use winit::keyboard::KeyCode;

pub trait Action {}

pub enum MoveAction {
    FORWARDS,
    BACKWARDS,
    LEFT,
    RIGHT,
}

pub struct KeyBindings {
    binding: HashMap<Vec<KeyCode>, Box<dyn Action>>,
}

impl KeyBindings {
    fn initialize() -> Self {
        Self {
            binding: HashMap::new(),
        }
    }

    pub fn add_binding(&mut self, key_bindings: Vec<KeyCode>, action: Box<dyn Action>) {
        self.binding.insert(key_bindings, action);
    }

    pub fn change_binding(&mut self, previous_bindings: Vec<KeyCode>, new_binding: Vec<KeyCode>) {
        if let Some(action) = self.remove_binding(previous_bindings) {
            self.add_binding(new_binding, action);
        } else {
            trace!("Previous Binding did not exist! Binding new binding to action");
        }
    }

    pub fn remove_binding(&mut self, previous_bindings: Vec<KeyCode>) -> Option<Box<dyn Action>> {
        self.binding.remove(&previous_bindings)
    }
}
