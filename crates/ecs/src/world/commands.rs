use crate::EntityId;

#[derive(Debug, PartialEq, Clone)]
pub enum EntityCommand {
    Spawn,
    Despawn(EntityId),
}
