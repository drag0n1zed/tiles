mod game;
mod screens;
mod timer;

use std::time::Duration;

use color_eyre::eyre::Result;
use ratatui::crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{DefaultTerminal, crossterm::event::Event};

use crate::{
    screens::{Screen, ScreenAction, menu::MenuScreen},
    timer::Timer,
};

fn main() -> Result<()> {
    color_eyre::install()?;

    ratatui::run(|terminal| -> Result<()> { App::new().run(terminal) })?;

    Ok(())
}

struct App {
    current_screen: Box<dyn Screen>,
    tick_timer: Timer,
    exit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            current_screen: Box::new(MenuScreen::default()),
            tick_timer: Timer::new(Duration::from_secs_f64(1.0 / 120.0)), // 120 TPS/FPS
            exit: false,
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        terminal.clear()?;

        while !self.exit {
            terminal.draw(|frame| self.current_screen.render_screen(frame))?;

            if event::poll(self.tick_timer.time_until_ready())? {
                while event::poll(Duration::ZERO)? {
                    let event = event::read()?;
                    if let Event::Key(KeyEvent { code, modifiers, .. }) = event {
                        match (code, modifiers) {
                            (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                                self.exit = true;
                            }
                            _ => {}
                        }
                    }
                    let action = self.current_screen.handle_input(event);
                    self.handle_action(action);
                }
            }

            let action = self.current_screen.update();
            self.handle_action(action);
        }
        Ok(())
    }

    fn handle_action(&mut self, action: ScreenAction) {
        match action {
            ScreenAction::ChangeScreen(screen) => self.current_screen = screen,
            _ => {}
        }
    }
}
