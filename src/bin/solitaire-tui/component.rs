use ratatui::{prelude::Rect, Frame};

use crate::{error::Result, event::Event};

pub mod app;
pub mod game;

pub trait Component {
    fn handle_event(&mut self, event: &Event) -> Result<()>;

    fn render(&self, f: &mut Frame, rect: Rect);
}
