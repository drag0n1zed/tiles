pub mod game;
pub mod menu;

use ratatui::crossterm::event::Event;
use ratatui::prelude::*;

#[derive(Default)]
pub enum ScreenAction {
    #[default]
    Nothing,
    ChangeScreen(Box<dyn Screen>),
}

pub trait Screen {
    fn handle_input(&mut self, event: Event) -> ScreenAction;
    fn update(&mut self) -> ScreenAction;
    fn render_screen(&self, frame: &mut Frame);
}
