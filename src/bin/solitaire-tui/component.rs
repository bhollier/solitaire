use ratatui::{prelude::Rect, Frame};

use crate::{
    error::Result,
    event::{Event, EventResult},
};

pub mod app;
pub mod game;

pub trait Component {
    fn handle_event(&mut self, event: &Event) -> EventResult;

    fn handle_tick(&mut self, dt: &std::time::Duration) -> Result<()>;

    fn render(&self, f: &mut Frame, rect: Rect);
}
