use crate::{CommandBuffer, EntityCommand, EntityId, World};

#[test]
fn test_empty_command_buffer_behavior() {
    let buffer = CommandBuffer::<EntityCommand>::new(Vec::default());

    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    assert_eq!(buffer.iter().count(), 0);
}

#[test]
fn test_command_buffer_preserves_insertion_order() {
    let mut buffer = CommandBuffer::new(Vec::default());

    buffer.push(EntityCommand::Despawn(EntityId::new_uuid(0, 0)));
    buffer.push(EntityCommand::Despawn(EntityId::new_uuid(1, 0)));
    buffer.push(EntityCommand::Despawn(EntityId::new_uuid(2, 0)));

    let values = buffer
        .iter()
        .map(|command| match command {
            entity_command => match entity_command {
                EntityCommand::Despawn(id) => id.index(),
                _ => unreachable!(),
            },
        })
        .collect::<Vec<_>>();

    assert_eq!(values, vec![0, 1, 2]);
}

#[test]
fn test_command_buffer_is_cleared_after_apply() {
    let mut buffer = CommandBuffer::new(Vec::default());

    buffer.push(EntityCommand::Spawn);
    buffer.push(EntityCommand::Spawn);

    World::default().apply_command_buffer(&mut buffer);

    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
}
