use std::collections::HashMap;

use anyhow::{Result, anyhow};

use crate::{Entity, EntityId};

pub struct EntityAllocator {
    entities: Vec<EntityId>,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self { entities: vec![] }
    }

    pub fn spawn(&self) -> Entity {
        let entity = Entity::new_uuid(idx, version);
        entity
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
        self.entities.len() - 1
    }
}
