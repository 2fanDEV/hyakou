use std::collections::{HashMap, HashSet};

use smallvec::SmallVec;

use crate::renderer::{actions::Action, types::mouse_delta::MouseButton};

static MAX_MOUSE_BUTTON_PRESSED_COUNT: usize = 5;

#[derive(Default, Debug)]
pub struct MouseBinding {
    buttons: SmallVec<[MouseButton; MAX_MOUSE_BUTTON_PRESSED_COUNT]>,
}

#[derive(Default, Debug)]
pub struct MouseBindingMap {
    bindings: HashMap<MouseBinding, Action>,
}

#[derive(Debug)]
pub struct MouseHandler {
    pressed_buttons: HashSet<MouseButton>,
}
