use std::{collections::HashMap, hash::Hash};

use log::{trace, warn};

use crate::renderer::actions::Action;

#[derive(Debug)]
pub(super) struct BindingMap<B> {
    bindings: HashMap<B, Action>,
}

impl<B> Default for BindingMap<B> {
    fn default() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }
}

impl<B> BindingMap<B>
where
    B: Eq + Hash,
{
    pub(super) fn from_entries(entries: impl IntoIterator<Item = (B, Action)>) -> Self {
        Self {
            bindings: entries.into_iter().collect(),
        }
    }

    pub(super) fn add_binding(&mut self, binding: B, action: Action) {
        if self.get_binding(&binding).is_some() {
            warn!("The binding is already in use!");
        } else {
            self.bindings.insert(binding, action);
        }
    }

    pub(super) fn change_binding(&mut self, previous_binding: B, new_binding: B) {
        if let Some(action) = self.remove_binding(&previous_binding) {
            self.add_binding(new_binding, action);
        } else {
            trace!("Previous Binding did not exist! Binding new binding to action");
        }
    }

    pub(super) fn get_binding(&self, binding: &B) -> Option<&Action> {
        self.bindings.get(binding)
    }

    pub(super) fn remove_binding(&mut self, binding: &B) -> Option<Action> {
        self.bindings.remove(binding)
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (&B, &Action)> {
        self.bindings.iter()
    }
}
