use crate::CommandBuffer;

#[derive(Debug, PartialEq)]
struct TestCommand(u32);

#[test]
fn test_empty_command_buffer_behavior() {
    let buffer = CommandBuffer::<TestCommand>::new(Vec::default());

    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    assert_eq!(buffer.iter().count(), 0);
}

#[test]
fn test_command_buffer_preserves_insertion_order() {
    let mut buffer = CommandBuffer::new(Vec::default());

    buffer.push(TestCommand(1));
    buffer.push(TestCommand(2));
    buffer.push(TestCommand(3));

    let values = buffer.iter().map(|command| command.0).collect::<Vec<_>>();

    assert_eq!(values, vec![1, 2, 3]);
}

#[test]
fn test_command_buffer_can_be_cleared_after_apply() {
    let mut buffer = CommandBuffer::new(Vec::default());

    buffer.push(TestCommand(1));
    buffer.clear();

    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
}
