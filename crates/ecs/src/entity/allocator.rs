use anyhow::{Result, anyhow};

use crate::EntityId;

#[derive(Debug)]
pub struct EntityAllocator {
    entities: Vec<EntityId>,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self { entities: vec![] }
    }

    pub fn spawn(&mut self) -> EntityId {
        let stale_entities = self.entities.iter().filter(|id| id.stale == true);
        let mut push_entity_id = false;
        let idx = if stale_entities.clone().count() > 0 {
            stale_entities
                .take(1)
                .next()
                .map(|id| (id.index, id.version + 1))
                .unwrap()
        } else {
            push_entity_id = true;
            (self.entities.len(), 0)
        };
        let new_entity = EntityId::new_uuid(idx.0, idx.1);
        if push_entity_id {
            self.entities.push(new_entity.clone());
        } else {
            self.entities[idx.0] = new_entity.clone();
        }
        new_entity
    }

    pub fn despawn(&mut self, id: EntityId) -> Result<()> {
        let position = self.entities.iter().position(|p| id.eq(p));
        match position {
            Some(idx) => {
                let Some(entity) = self.entities.get_mut(idx) else {
                    return Err(anyhow!("No entity at index `{:?}` found", idx));
                };
                entity.stale = true;
                Ok(())
            }
            None => Ok(()),
        }
    }

    pub fn get_next_idx(&self) -> usize {
        let length = self.entities.len();
        if length > 0 { length - 1 } else { 0 }
    }
}
