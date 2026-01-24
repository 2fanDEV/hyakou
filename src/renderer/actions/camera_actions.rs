#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraActions {
    SlowModifier,
    SpeedModifier,
    Forwards,
    Backwards,
    Left,
    Right,
    Up,
    Down,
}
