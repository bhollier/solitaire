use solitaire::ui;

fn convert_key_code(key_code: ratzilla::event::KeyCode) -> ui::event::KeyCode {
    match key_code {
        ratzilla::event::KeyCode::Backspace => ui::event::KeyCode::Backspace,
        ratzilla::event::KeyCode::Enter => ui::event::KeyCode::Enter,
        ratzilla::event::KeyCode::Left => ui::event::KeyCode::Left,
        ratzilla::event::KeyCode::Right => ui::event::KeyCode::Right,
        ratzilla::event::KeyCode::Up => ui::event::KeyCode::Up,
        ratzilla::event::KeyCode::Down => ui::event::KeyCode::Down,
        ratzilla::event::KeyCode::Home => ui::event::KeyCode::Home,
        ratzilla::event::KeyCode::End => ui::event::KeyCode::End,
        ratzilla::event::KeyCode::PageUp => ui::event::KeyCode::PageUp,
        ratzilla::event::KeyCode::PageDown => ui::event::KeyCode::PageDown,
        ratzilla::event::KeyCode::Tab => ui::event::KeyCode::Tab,
        ratzilla::event::KeyCode::Delete => ui::event::KeyCode::Delete,
        ratzilla::event::KeyCode::F(n) => ui::event::KeyCode::F(n),
        ratzilla::event::KeyCode::Char(c) => ui::event::KeyCode::Char(c),
        ratzilla::event::KeyCode::Esc => ui::event::KeyCode::Esc,
        _ => ui::event::KeyCode::Unknown,
    }
}

pub fn convert_key_event(event: ratzilla::event::KeyEvent) -> ui::event::Event {
    ui::event::Event::KeyPress(
        convert_key_code(event.code),
        ui::event::Modifiers {
            ctrl: event.ctrl,
            alt: event.alt,
            shift: event.shift,
        },
    )
}

pub fn convert_mouse_event(event: ratzilla::event::MouseEvent) -> ui::event::Event {
    match event {
        ratzilla::event::MouseEvent {
            event: ratzilla::event::MouseEventKind::Pressed,
            ..
        } => ui::event::Event::MousePress(
            // todo these values are hard coded in Ratzilla,
            //  ideally we'd infer it from the size of the canvas
            //  vs the size of the terminal in cells
            (event.x / 10) as u16,
            (event.y / 19) as u16,
            ui::event::Modifiers {
                ctrl: event.ctrl,
                alt: event.alt,
                shift: event.shift,
            },
        ),
        _ => ui::event::Event::Unknown,
    }
}
