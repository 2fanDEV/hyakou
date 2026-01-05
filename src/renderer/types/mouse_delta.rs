#[derive(Default, Debug, PartialEq, Eq)]
pub enum MouseButton {
    #[default]
    Right,
    Left,
    Middle,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub enum MouseAction {
    Clicked,
    Released,
    #[default]
    NoAction,
}

#[derive(Default, Debug)]
pub struct MovementDelta {
    x: f64,
    y: f64,
}

impl MovementDelta {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}

#[derive(Default, Debug, Clone)]
pub struct MousePosition {
    x: f64,
    y: f64,
}

impl MousePosition {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}

#[derive(Default, Debug)]
pub struct MouseDelta {
    pub delta_position: MovementDelta,
    pub state: MouseState,
    pub is_mouse_on_window: bool,
    pub position: MousePosition,
}

#[derive(Default, Debug)]
pub struct MouseState {
    button: MouseButton,
    action: MouseAction,
}

impl MouseDelta {
    pub fn set_is_mouse_on_window(&mut self, val: bool) {
        self.is_mouse_on_window = val;
    }

    pub fn is_mouse_on_window(&self) -> bool {
        self.is_mouse_on_window
    }
}

impl MouseState {
    pub fn new(button: MouseButton, action: MouseAction) -> Self {
        Self { button, action }
    }

    pub fn get_button(&self) -> &MouseButton {
        &self.button
    }

    pub fn get_action(&self) -> &MouseAction {
        &self.action
    }
}
