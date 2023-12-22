use std::{sync, thread};

use crossterm::event;

#[derive(Copy, Clone)]
pub enum Event {
    KeyPress(event::KeyCode, event::KeyModifiers),
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
            _ => Event::Unknown,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Message {
    Event(Event),
    Tick,
}

pub struct Events {
    rx: sync::mpsc::Receiver<Message>,
}

impl Events {
    pub fn new(tick_rate: u64) -> Events {
        let (tx, rx) = sync::mpsc::channel();

        {
            let tx = tx.clone();
            thread::spawn(move || loop {
                if event::poll(std::time::Duration::from_millis(tick_rate)).unwrap() {
                    let event = Event::from(event::read().unwrap());
                    match tx.send(Message::Event(event)) {
                        Ok(_) => {}
                        // If error then assume the receiver has shutdown and just exit
                        Err(_) => break,
                    }
                }
                match tx.send(Message::Tick) {
                    Ok(_) => {}
                    Err(_) => break,
                }
            });
        }

        Events { rx }
    }

    pub fn next(&self) -> Result<Message, sync::mpsc::RecvError> {
        self.rx.recv()
    }
}
