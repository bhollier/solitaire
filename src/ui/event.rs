use crate::ui::error::Error;

#[derive(Copy, Clone)]
pub enum KeyCode {
    Char(char),
    F(u8),
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Tab,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Esc,
    Unknown,
}

#[derive(Copy, Clone)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

#[derive(Copy, Clone)]
pub enum Event {
    KeyPress(KeyCode, Modifiers),
    MousePress(u16, u16, Modifiers),
    Unknown,
}

#[derive(Copy, Clone)]
pub enum EventState {
    Consumed,
    NotConsumed,
}

pub type EventResult = Result<EventState, Error>;
