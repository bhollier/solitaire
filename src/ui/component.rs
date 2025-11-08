use ratatui::{prelude::Rect, Frame};

use crate::{
    ui::error::Result,
    ui::event::{Event, EventResult},
};

pub mod app;
pub mod game;

pub trait Component {
    fn handle_event(&mut self, event: &Event) -> EventResult;

    fn handle_tick(&mut self, dt: &web_time::Duration) -> Result<()>;

    fn render(&mut self, f: &mut Frame, rect: Rect);
}
