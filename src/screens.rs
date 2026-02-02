pub mod game_screen;

use ratatui::crossterm::event::Event;
use ratatui::prelude::*;

pub enum ScreenAction {
    Nothing,
    Quit,
    ChangeScreen(Box<dyn Screen>),
}
pub trait Screen {
    fn handle_input(&mut self, event: Event);
    fn update(&mut self) -> ScreenAction;
    fn render_screen(&self, frame: &mut Frame);
}
