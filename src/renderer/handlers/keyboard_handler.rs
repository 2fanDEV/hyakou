use std::ops::Index;

use log::{error, trace};

use crate::renderer::types::keys::Key;

#[derive(Debug, Default, PartialEq, Eq)]
pub enum KeyAction {
    PRESSED,
    HELD,
    #[default]
    RELEASED,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct KeyState {
    key: Key,
    action: KeyAction,
}

pub struct KeyboardHandler {
    keys: Vec<KeyState>,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self { keys: vec![] }
    }

    pub fn handle_key_state(&mut self, key_state: KeyState) {
        if key_state.action.eq(&KeyAction::RELEASED) {
            match self.keys.binary_search_by(|a| a.key.cmp(&key_state.key)) {
                Ok(idx) => self.remove_key_by_idx(idx),
                Err(e) => {
                    error!("Key {:?} was not pressed before.", key_state.key);
                    trace!(
                        "Key {:?} was not pressed before. Full Error Message: {:?}",
                        key_state.key, e
                    );
                }
            };
        } else {

    }

    fn remove_key_by_idx(&mut self, idx: usize) {
        self.keys.remove(idx);
    }
}
