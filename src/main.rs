mod game;
mod screens;
mod timer;

use std::time::Duration;

use crate::{
    screens::{Screen, ScreenAction, menu::MenuScreen},
    timer::Timer,
};
use color_eyre::eyre::Result;
use ratatui::crossterm::event::{self, KeyCode, KeyModifiers};
use ratatui::{DefaultTerminal, crossterm::event::Event};

fn main() -> Result<()> {
    color_eyre::install()?;

    ratatui::run(|terminal| -> Result<()> { App::new().run(terminal) })?;

    Ok(())
}

struct App {
    screen_stack: Vec<Box<dyn Screen>>,
    tick_timer: Timer,
}

impl App {
    fn new() -> Self {
        Self {
            screen_stack: vec![Box::new(MenuScreen::main_menu())],
            tick_timer: Timer::new(Duration::from_secs_f64(1.0 / 120.0)), // 120 TPS/FPS
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        terminal.clear()?;

        loop {
            if let Some(screen) = self.screen_stack.last() {
                terminal.draw(|frame| screen.render_screen(frame))?;
            } else {
                break;
            }

            let mut input = None;

            if event::poll(self.tick_timer.time_until_ready())? {
                while event::poll(Duration::ZERO)? {
                    if let Event::Key(key) = event::read()? {
                        match (key.modifiers, key.code) {
                            (KeyModifiers::CONTROL, KeyCode::Char('c')) => self.screen_stack.clear(),
                            (_, KeyCode::Esc) => {
                                self.screen_stack.pop();
                            }
                            _ => {}
                        }
                        input = Some(key);
                    };
                }
            }

            if let Some(screen) = self.screen_stack.last_mut() {
                let action = screen.update(input)?;
                self.handle_action(action);
            }
        }
        Ok(())
    }

    fn handle_action(&mut self, action: ScreenAction) {
        match action {
            ScreenAction::PushScreen(screen) => self.screen_stack.push(screen),
            ScreenAction::PopScreen => {
                self.screen_stack.pop();
            }
            _ => {}
        }
    }
}
