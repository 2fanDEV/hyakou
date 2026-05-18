use crate::EntityId;

pub enum EntityCommand {
    Spawn,
    Despawn(EntityId),
}
