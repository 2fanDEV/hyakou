use std::{cmp::Ordering, vec};

use shared::{Shared, SharedAccess};

use crate::{Component, Components, EntityAllocator, EntityId};

/// Read-only single-component query.
///
/// Results are deterministic: live entities are returned by ascending entity index, then version.
pub struct Query<'a, C: Component> {
    entries: vec::IntoIter<(&'a EntityId, &'a C)>,
}

/// Mutable single-component query.
///
/// Results use the same deterministic order as [`Query`]. Rust's borrow rules prevent overlapping
/// mutable queries from the same world.
pub struct QueryMut<'a, C: Component> {
    entries: vec::IntoIter<(&'a EntityId, &'a mut C)>,
}

impl<'a, C: Component> Query<'a, C> {
    pub(crate) fn new(components: &'a Components, allocator: &'a Shared<EntityAllocator>) -> Self {
        let mut entries = Vec::new();
        if let Some(components) = components.iter::<C>() {
            for (entity, component) in components {
                if is_alive(allocator, entity) {
                    entries.push((entity, component));
                }
            }
        }
        sort_by_entity(&mut entries);

        Self {
            entries: entries.into_iter(),
        }
    }
}

impl<'a, C: Component> QueryMut<'a, C> {
    pub(crate) fn new(
        components: &'a mut Components,
        allocator: &'a Shared<EntityAllocator>,
    ) -> Self {
        let mut entries = Vec::new();
        if let Some(components) = components.iter_mut::<C>() {
            for (entity, component) in components {
                if is_alive(allocator, entity) {
                    entries.push((entity, component));
                }
            }
        }
        sort_by_entity(&mut entries);

        Self {
            entries: entries.into_iter(),
        }
    }
}

impl<'a, C: Component> Iterator for Query<'a, C> {
    type Item = (&'a EntityId, &'a C);

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next()
    }
}

impl<'a, C: Component> Iterator for QueryMut<'a, C> {
    type Item = (&'a EntityId, &'a mut C);

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.next()
    }
}

fn is_alive(allocator: &Shared<EntityAllocator>, entity: &EntityId) -> bool {
    allocator
        .try_read_shared(|alloc| alloc.is_alive(entity))
        .unwrap_or(false)
}

fn sort_by_entity<T>(entries: &mut [(&EntityId, T)]) {
    entries.sort_by(|(left, _), (right, _)| compare_entity(left, right));
}

fn compare_entity(left: &EntityId, right: &EntityId) -> Ordering {
    left.index()
        .cmp(&right.index())
        .then_with(|| left.version().cmp(&right.version()))
}
