use shared_types::id::Id;

use crate::{Command, CommandBuffer, Entity, EntityId, World, WorldCommand};

#[test]
fn test_empty_command_buffer_behavior() {
    let buffer = CommandBuffer::new(Vec::default());

    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    assert_eq!(buffer.iter().count(), 0);
}

#[test]
fn test_command_buffer_preserves_insertion_order() {
    let mut buffer = CommandBuffer::new(Vec::default());

    buffer.push(Command::World(WorldCommand::DESPAWN(EntityId::new_uuid(
        0, 0,
    ))));
    buffer.push(Command::World(WorldCommand::DESPAWN(EntityId::new_uuid(
        1, 0,
    ))));
    buffer.push(Command::World(WorldCommand::DESPAWN(EntityId::new_uuid(
        2, 0,
    ))));

    let values = buffer
        .iter()
        .map(|command| match command {
            Command::World(val) => match val {
                WorldCommand::DESPAWN(id) => id.index(),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();

    assert_eq!(values, vec![0, 1, 2]);
}

#[test]
fn test_command_buffer_can_be_cleared_after_apply() {
    let mut buffer = CommandBuffer::new(Vec::default());

    buffer.push(Command::World(WorldCommand::SPAWN));
    buffer.push(Command::World(WorldCommand::DESPAWN(EntityId::new_uuid(
        0, 0,
    ))));

    let mut world = World::default();
    world.apply_command_buffer(&mut buffer);
    buffer.clear();

    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
}

#[test]
fn test_command_buffer_empty() {
    let mut buffer = CommandBuffer::new(Vec::default());
    let mut world = World::default();
    world.apply_command_buffer(&mut buffer);
}
