use std::{sync, thread, time::Instant};

use crossterm::event;

use crate::error::Error;

#[derive(Copy, Clone)]
pub enum Event {
    KeyPress(event::KeyCode, event::KeyModifiers),
    MousePress(u16, u16, event::KeyModifiers),
    Unknown,
}

impl From<event::Event> for Event {
    fn from(event: event::Event) -> Self {
        match event {
            event::Event::Key(key_event) => match key_event {
                event::KeyEvent {
                    kind: event::KeyEventKind::Press,
                    ..
                } => Event::KeyPress(key_event.code, key_event.modifiers),
                _ => Event::Unknown,
            },
            event::Event::Mouse(mouse_event) => match mouse_event {
                event::MouseEvent {
                    kind: event::MouseEventKind::Down(_),
                    ..
                } => Event::MousePress(mouse_event.column, mouse_event.row, mouse_event.modifiers),
                _ => Event::Unknown,
            },
            _ => Event::Unknown,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Message {
    Event(Event),
    Tick(std::time::Duration),
}

#[derive(Copy, Clone)]
pub enum EventState {
    Consumed,
    NotConsumed,
}

pub type EventResult = Result<EventState, Error>;

pub struct Events {
    rx: sync::mpsc::Receiver<Message>,
}

impl Events {
    pub fn new(tick_rate: std::time::Duration) -> Events {
        let (tx, rx) = sync::mpsc::channel();

        {
            let tx = tx.clone();
            let mut last_tick_instant = Instant::now();
            thread::spawn(move || loop {
                let timeout =
                    (last_tick_instant + tick_rate).saturating_duration_since(Instant::now());

                if event::poll(timeout).unwrap() {
                    loop {
                        let event = Event::from(event::read().unwrap());
                        match event {
                            Event::Unknown => {}
                            event => {
                                match tx.send(Message::Event(event)) {
                                    Ok(_) => {}
                                    // If error then assume the receiver has shutdown and just exit
                                    Err(_) => break,
                                }
                            }
                        };
                        // Keep polling only for events that are immediately available
                        if !event::poll(std::time::Duration::from_millis(0)).unwrap() {
                            break;
                        }
                    }
                }
                let now = Instant::now();
                let dt = now.duration_since(last_tick_instant);
                if dt >= tick_rate {
                    last_tick_instant = now;
                    match tx.send(Message::Tick(dt)) {
                        Ok(_) => {}
                        Err(_) => break,
                    }
                }
            });
        }

        Events { rx }
    }

    pub fn next(&self) -> Result<Message, sync::mpsc::RecvError> {
        self.rx.recv()
    }
}
