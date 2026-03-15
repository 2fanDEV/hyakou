#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CameraActions {
    SlowModifier,
    SpeedModifier,
    Forwards,
    Backwards,
    Left,
    Right,
    Up,
    Down,
    Drag,
}
