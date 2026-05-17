use crate::EntityId;

#[derive(Debug)]
struct EntitySlot {
    version: usize,
    alive: bool,
}

#[derive(Debug, Default)]
pub struct EntityAllocator {
    slots: Vec<EntitySlot>,
    free_slots: Vec<usize>,
}

impl EntityAllocator {
    pub fn spawn(&mut self) -> EntityId {
        if let Some(index) = self.free_slots.pop() {
            let slot = &mut self.slots[index];
            slot.version += 1;
            slot.alive = true;
            return EntityId::new_uuid(index, slot.version);
        }

        let index = self.slots.len();
        self.slots.push(EntitySlot {
            version: 0,
            alive: true,
        });

        EntityId::new_uuid(index, 0)
    }

    pub fn despawn(&mut self, id: &EntityId) -> bool {
        let Some(slot) = self.slots.get_mut(id.index()) else {
            return false;
        };

        if !slot.alive || slot.version != id.version() {
            return false;
        }

        slot.alive = false;
        self.free_slots.push(id.index());

        true
    }

    pub fn is_alive(&self, id: &EntityId) -> bool {
        self.slots
            .get(id.index())
            .is_some_and(|slot| slot.alive && slot.version == id.version())
    }

    pub fn get_next_idx(&self) -> usize {
        let length = self.slots.len();
        if length > 0 { length - 1 } else { 0 }
    }

    pub fn alive_count(&self) -> usize {
        self.slots.iter().filter(|s| s.alive).count()
    }
}

#[cfg(test)]
#[path = "tests/allocator_tests.rs"]
mod allocator_tests;
