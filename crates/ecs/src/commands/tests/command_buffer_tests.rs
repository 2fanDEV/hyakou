use crate::{CommandBuffer, EntityId, World, world::commands::EntityCommand};

#[test]
fn test_empty_command_buffer_behavior() {
    let buffer = CommandBuffer::<EntityCommand>::new(vec![]);

    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    assert_eq!(buffer.iter().count(), 0);
}

#[test]
fn test_command_buffer_preserves_insertion_order() {
    let mut buffer = CommandBuffer::new(vec![]);

    buffer.push(EntityCommand::Despawn(EntityId::new_uuid(0, 0)));
    buffer.push(EntityCommand::Despawn(EntityId::new_uuid(1, 0)));
    buffer.push(EntityCommand::Despawn(EntityId::new_uuid(2, 0)));

    let values = buffer
        .iter()
        .map(|command| match command {
            EntityCommand::Despawn(id) => id.index(),
            EntityCommand::Spawn => unreachable!(),
        })
        .collect::<Vec<_>>();

    assert_eq!(values, vec![0, 1, 2]);
}

#[test]
fn test_command_buffer_is_cleared_after_apply() {
    let mut world = World::default();
    world.recorder().spawn();
    world.recorder().spawn();
    world.apply_command_buffer();
    assert!(world.buffer().is_empty());
}
