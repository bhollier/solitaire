use std::{sync, thread, time::Instant};

use solitaire::ui;

#[derive(Copy, Clone)]
pub enum Message {
    Event(ui::event::Event),
    Tick(web_time::Duration),
}

fn convert_key_code(key_code: crossterm::event::KeyCode) -> ui::event::KeyCode {
    match key_code {
        crossterm::event::KeyCode::Backspace => ui::event::KeyCode::Backspace,
        crossterm::event::KeyCode::Enter => ui::event::KeyCode::Enter,
        crossterm::event::KeyCode::Left => ui::event::KeyCode::Left,
        crossterm::event::KeyCode::Right => ui::event::KeyCode::Right,
        crossterm::event::KeyCode::Up => ui::event::KeyCode::Up,
        crossterm::event::KeyCode::Down => ui::event::KeyCode::Down,
        crossterm::event::KeyCode::Home => ui::event::KeyCode::Home,
        crossterm::event::KeyCode::End => ui::event::KeyCode::End,
        crossterm::event::KeyCode::PageUp => ui::event::KeyCode::PageUp,
        crossterm::event::KeyCode::PageDown => ui::event::KeyCode::PageDown,
        crossterm::event::KeyCode::Tab => ui::event::KeyCode::Tab,
        crossterm::event::KeyCode::BackTab => ui::event::KeyCode::Tab,
        crossterm::event::KeyCode::Delete => ui::event::KeyCode::Delete,
        crossterm::event::KeyCode::F(n) => ui::event::KeyCode::F(n),
        crossterm::event::KeyCode::Char(c) => ui::event::KeyCode::Char(c),
        crossterm::event::KeyCode::Esc => ui::event::KeyCode::Esc,
        _ => ui::event::KeyCode::Unknown,
    }
}

fn convert_modifiers(modifiers: crossterm::event::KeyModifiers) -> ui::event::Modifiers {
    ui::event::Modifiers {
        shift: modifiers.contains(crossterm::event::KeyModifiers::SHIFT),
        ctrl: modifiers.contains(crossterm::event::KeyModifiers::CONTROL),
        alt: modifiers.contains(crossterm::event::KeyModifiers::ALT),
    }
}

fn convert_event(event: crossterm::event::Event) -> ui::event::Event {
    match event {
        crossterm::event::Event::Key(key_event) => match key_event {
            crossterm::event::KeyEvent {
                kind: crossterm::event::KeyEventKind::Press,
                ..
            } => ui::event::Event::KeyPress(
                convert_key_code(key_event.code),
                convert_modifiers(key_event.modifiers),
            ),
            _ => ui::event::Event::Unknown,
        },
        crossterm::event::Event::Mouse(mouse_event) => match mouse_event {
            crossterm::event::MouseEvent {
                kind: crossterm::event::MouseEventKind::Down(_),
                ..
            } => ui::event::Event::MousePress(
                mouse_event.column,
                mouse_event.row,
                convert_modifiers(mouse_event.modifiers),
            ),
            _ => ui::event::Event::Unknown,
        },
        _ => ui::event::Event::Unknown,
    }
}

pub struct Events {
    rx: sync::mpsc::Receiver<Message>,
}

impl Events {
    pub fn new(tick_rate: web_time::Duration) -> Events {
        let (tx, rx) = sync::mpsc::channel();

        {
            let tx = tx.clone();
            let mut last_tick_instant = Instant::now();
            thread::spawn(move || loop {
                let timeout =
                    (last_tick_instant + tick_rate).saturating_duration_since(Instant::now());

                if crossterm::event::poll(timeout).unwrap() {
                    loop {
                        let event = convert_event(crossterm::event::read().unwrap());
                        match event {
                            ui::event::Event::Unknown => {}
                            event => {
                                match tx.send(Message::Event(event)) {
                                    Ok(_) => {}
                                    // If error then assume the receiver has shutdown and just exit
                                    Err(_) => break,
                                }
                            }
                        };
                        // Keep polling only for events that are immediately available
                        if !crossterm::event::poll(web_time::Duration::from_millis(0)).unwrap() {
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
