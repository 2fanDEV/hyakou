use crate::EntityAllocator;

#[test]
fn test_spawn_creates_fresh_entity_ids() {
    let mut allocator = EntityAllocator::new();

    let first = allocator.spawn();
    let second = allocator.spawn();

    assert_eq!(first.index(), 0);
    assert_eq!(first.version(), 0);
    assert_eq!(second.index(), 1);
    assert_eq!(second.version(), 0);
    assert!(allocator.is_alive(&first));
    assert!(allocator.is_alive(&second));
}

#[test]
fn test_despawn_marks_entity_dead() {
    let mut allocator = EntityAllocator::new();
    let entity = allocator.spawn();

    assert!(allocator.despawn(entity.clone()));

    assert!(!allocator.is_alive(&entity));
}

#[test]
fn test_reusing_slot_increments_generation() {
    let mut allocator = EntityAllocator::new();
    let old_entity = allocator.spawn();

    assert!(allocator.despawn(old_entity.clone()));
    let new_entity = allocator.spawn();

    assert_eq!(new_entity.index(), old_entity.index());
    assert_eq!(new_entity.version(), old_entity.version() + 1);
    assert!(!allocator.is_alive(&old_entity));
    assert!(allocator.is_alive(&new_entity));
}

#[test]
fn test_repeated_spawn_despawn_cycles() {
    let mut allocator = EntityAllocator::new();
    let mut entity = allocator.spawn();

    for expected_version in 1..=8 {
        assert!(allocator.despawn(entity));
        entity = allocator.spawn();

        assert_eq!(entity.index(), 0);
        assert_eq!(entity.version(), expected_version);
        assert!(allocator.is_alive(&entity));
    }
}

#[test]
fn test_dead_or_stale_entities_fail_safely() {
    let mut allocator = EntityAllocator::new();
    let stale_entity = allocator.spawn();

    assert!(allocator.despawn(stale_entity.clone()));
    assert!(!allocator.despawn(stale_entity.clone()));

    let new_entity = allocator.spawn();

    assert!(!allocator.despawn(stale_entity));
    assert!(allocator.despawn(new_entity));
}
