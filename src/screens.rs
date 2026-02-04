pub mod file_picker;
pub mod game;
pub mod menu;

use color_eyre::eyre::Result;
use ratatui::crossterm::event::KeyEvent;
use ratatui::prelude::*;

#[derive(Default)]
pub enum ScreenAction {
    #[default]
    Nothing,
    PushScreen(Box<dyn Screen>),
    PopScreen,
}

impl<T: Screen + 'static> From<T> for ScreenAction {
    fn from(screen: T) -> Self {
        ScreenAction::PushScreen(Box::new(screen))
    }
}

pub trait Screen {
    fn update(&mut self, key: Option<KeyEvent>) -> Result<ScreenAction>;
    fn render_screen(&self, frame: &mut Frame);
}
