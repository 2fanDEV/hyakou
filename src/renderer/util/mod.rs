use std::path::PathBuf;

use bytemuck::bytes_of;
use glam::Mat4;
use winit::keyboard::KeyCode;

pub type Width = u32;
pub type Height = u32;

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: Width,
    pub height: Height,
}

pub trait Concatable {
    fn concat(&mut self, text: &str) -> &str;
}

impl Concatable for String {
    fn concat(&mut self, text: &str) -> &str {
        self.push_str(text);
        self.as_str()
    }
}

pub fn get_matrix_as_bytes(mat: &Mat4) -> &[u8] {
    bytes_of(mat)
}

pub fn get_relative_path() -> PathBuf {
    let path = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(path)
}

pub fn keycode_to_index(keycode: KeyCode) -> usize {
    match keycode {
        KeyCode::KeyA => 0,
        KeyCode::KeyB => 1,
        KeyCode::KeyC => 2,
        KeyCode::KeyD => 3,
        KeyCode::KeyE => 4,
        KeyCode::KeyF => 5,
        KeyCode::KeyG => 6,
        KeyCode::KeyH => 7,
        KeyCode::KeyI => 8,
        KeyCode::KeyJ => 9,
        KeyCode::KeyK => 10,
        KeyCode::KeyL => 11,
        KeyCode::KeyM => 12,
        KeyCode::KeyN => 13,
        KeyCode::KeyO => 14,
        KeyCode::KeyP => 15,
        KeyCode::KeyQ => 16,
        KeyCode::KeyR => 17,
        KeyCode::KeyS => 18,
        KeyCode::KeyT => 19,
        KeyCode::KeyU => 20,
        KeyCode::KeyV => 21,
        KeyCode::KeyW => 22,
        KeyCode::KeyX => 23,
        KeyCode::KeyY => 24,
        KeyCode::KeyZ => 25,

        // Numbers (26-35)
        KeyCode::Digit0 => 26,
        KeyCode::Digit1 => 27,
        KeyCode::Digit2 => 28,
        KeyCode::Digit3 => 29,
        KeyCode::Digit4 => 30,
        KeyCode::Digit5 => 31,
        KeyCode::Digit6 => 32,
        KeyCode::Digit7 => 33,
        KeyCode::Digit8 => 34,
        KeyCode::Digit9 => 35,

        // Function keys (36-47)
        KeyCode::F1 => 36,
        KeyCode::F2 => 37,
        KeyCode::F3 => 38,
        KeyCode::F4 => 39,
        KeyCode::F5 => 40,
        KeyCode::F6 => 41,
        KeyCode::F7 => 42,
        KeyCode::F8 => 43,
        KeyCode::F9 => 44,
        KeyCode::F10 => 45,
        KeyCode::F11 => 46,
        KeyCode::F12 => 47,

        // Arrow keys (48-51)
        KeyCode::ArrowUp => 48,
        KeyCode::ArrowDown => 49,
        KeyCode::ArrowLeft => 50,
        KeyCode::ArrowRight => 51,

        // Modifiers (52-59)
        KeyCode::ShiftLeft => 52,
        KeyCode::ShiftRight => 53,
        KeyCode::ControlLeft => 54,
        KeyCode::ControlRight => 55,
        KeyCode::AltLeft => 56,
        KeyCode::AltRight => 57,
        KeyCode::SuperLeft => 58, // Windows/Command key
        KeyCode::SuperRight => 59,

        // Common special keys (60-69)
        KeyCode::Space => 60,
        KeyCode::Enter => 61,
        KeyCode::Escape => 62,
        KeyCode::Backspace => 63,
        KeyCode::Tab => 64,
        KeyCode::CapsLock => 65,
        KeyCode::Delete => 66,
        KeyCode::Insert => 67,
        KeyCode::Home => 68,
        KeyCode::End => 69,

        // Page navigation (70-71)
        KeyCode::PageUp => 70,
        KeyCode::PageDown => 71,

        // Punctuation and symbols (72-87)
        KeyCode::Minus => 72,
        KeyCode::Equal => 73,
        KeyCode::BracketLeft => 74,
        KeyCode::BracketRight => 75,
        KeyCode::Backslash => 76,
        KeyCode::Semicolon => 77,
        KeyCode::Quote => 78,
        KeyCode::Comma => 79,
        KeyCode::Period => 80,
        KeyCode::Slash => 81,
        KeyCode::Backquote => 82,

        // Numpad (88-107)
        KeyCode::Numpad0 => 88,
        KeyCode::Numpad1 => 89,
        KeyCode::Numpad2 => 90,
        KeyCode::Numpad3 => 91,
        KeyCode::Numpad4 => 92,
        KeyCode::Numpad5 => 93,
        KeyCode::Numpad6 => 94,
        KeyCode::Numpad7 => 95,
        KeyCode::Numpad8 => 96,
        KeyCode::Numpad9 => 97,
        KeyCode::NumpadAdd => 98,
        KeyCode::NumpadSubtract => 99,
        KeyCode::NumpadMultiply => 100,
        KeyCode::NumpadDivide => 101,
        KeyCode::NumpadEnter => 102,
        KeyCode::NumpadDecimal => 103,
        KeyCode::NumpadEqual => 104,
        KeyCode::NumLock => 105,

        // Misc (106+)
        KeyCode::ScrollLock => 106,
        KeyCode::Pause => 107,
        KeyCode::PrintScreen => 108,
        KeyCode::ContextMenu => 109,

        // Anything else maps to 255 (reserved for unknown keys)
        _ => 255,
    }
}
