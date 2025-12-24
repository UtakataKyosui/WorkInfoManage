// Platform-agnostic key event handling
// This module provides a unified interface for keyboard input across Terminal and Web platforms

use std::fmt;

/// Platform-agnostic key code representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Esc,
    Backspace,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Tab,
    Delete,
    F(u8),
}

/// Platform-agnostic key modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl KeyModifiers {
    pub const NONE: Self = KeyModifiers {
        ctrl: false,
        alt: false,
        shift: false,
    };

    pub const CONTROL: Self = KeyModifiers {
        ctrl: true,
        alt: false,
        shift: false,
    };

    pub const ALT: Self = KeyModifiers {
        ctrl: false,
        alt: true,
        shift: false,
    };

    pub fn contains(&self, other: KeyModifiers) -> bool {
        (!other.ctrl || self.ctrl) && (!other.alt || self.alt) && (!other.shift || self.shift)
    }
}

/// Platform-agnostic key event
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    pub fn new(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }
}

// Conversion from crossterm KeyCode (Terminal)
#[cfg(not(target_arch = "wasm32"))]
impl From<crossterm::event::KeyEvent> for KeyEvent {
    fn from(event: crossterm::event::KeyEvent) -> Self {
        use crossterm::event::{KeyCode as CKC, KeyModifiers as CKM};

        let code = match event.code {
            CKC::Char(c) => KeyCode::Char(c),
            CKC::Enter => KeyCode::Enter,
            CKC::Esc => KeyCode::Esc,
            CKC::Backspace => KeyCode::Backspace,
            CKC::Left => KeyCode::Left,
            CKC::Right => KeyCode::Right,
            CKC::Up => KeyCode::Up,
            CKC::Down => KeyCode::Down,
            CKC::Home => KeyCode::Home,
            CKC::End => KeyCode::End,
            CKC::Tab => KeyCode::Tab,
            CKC::Delete => KeyCode::Delete,
            CKC::F(n) => KeyCode::F(n),
            _ => return Self::new(KeyCode::Char('\0')), // Unknown key
        };

        let modifiers = KeyModifiers {
            ctrl: event.modifiers.contains(CKM::CONTROL),
            alt: event.modifiers.contains(CKM::ALT),
            shift: event.modifiers.contains(CKM::SHIFT),
        };

        Self { code, modifiers }
    }
}

// Conversion from ratzilla KeyEvent (Web)
#[cfg(target_arch = "wasm32")]
impl From<ratzilla::event::KeyEvent> for KeyEvent {
    fn from(event: ratzilla::event::KeyEvent) -> Self {
        use ratzilla::event::KeyCode as RKC;

        let code = match event.code {
            RKC::Char(c) => KeyCode::Char(c),
            RKC::Enter => KeyCode::Enter,
            RKC::Esc => KeyCode::Esc,
            RKC::Backspace => KeyCode::Backspace,
            RKC::Left => KeyCode::Left,
            RKC::Right => KeyCode::Right,
            RKC::Up => KeyCode::Up,
            RKC::Down => KeyCode::Down,
            RKC::Home => KeyCode::Home,
            RKC::End => KeyCode::End,
            RKC::Tab => KeyCode::Tab,
            RKC::Delete => KeyCode::Delete,
            RKC::F(n) => KeyCode::F(n),
            _ => return Self::new(KeyCode::Char('\0')), // Unknown key
        };

        let modifiers = KeyModifiers {
            ctrl: event.ctrl,
            alt: event.alt,
            shift: event.shift,
        };

        Self { code, modifiers }
    }
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyCode::Char(c) => write!(f, "{}", c),
            KeyCode::Enter => write!(f, "Enter"),
            KeyCode::Esc => write!(f, "Esc"),
            KeyCode::Backspace => write!(f, "Backspace"),
            KeyCode::Left => write!(f, "←"),
            KeyCode::Right => write!(f, "→"),
            KeyCode::Up => write!(f, "↑"),
            KeyCode::Down => write!(f, "↓"),
            KeyCode::Home => write!(f, "Home"),
            KeyCode::End => write!(f, "End"),
            KeyCode::Tab => write!(f, "Tab"),
            KeyCode::Delete => write!(f, "Del"),
            KeyCode::F(n) => write!(f, "F{}", n),
        }
    }
}
