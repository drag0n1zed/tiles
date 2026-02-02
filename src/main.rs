mod game;
mod screens;
mod timer;

use std::{fs, time::Duration};

use color_eyre::eyre::Result;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event;

use crate::{
    game::logic::grid::Grid,
    screens::{Screen, ScreenAction, game_screen::GameScreen},
    timer::Timer,
};

fn main() -> Result<()> {
    color_eyre::install()?;
    // Import save file
    let grid = Grid::from_ron(fs::read_to_string("grid.ron")?.as_str())?;

    ratatui::run(|terminal| -> Result<()> { App::from_grid(grid).run(terminal) })?;

    Ok(())
}

struct App {
    current_screen: Box<dyn Screen>,
    tick_timer: Timer,
    exit: bool,
}

impl App {
    fn from_grid(grid: Grid) -> Self {
        Self {
            current_screen: Box::new(GameScreen::from_grid(grid)),
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
                    self.current_screen.handle_input(event::read()?);
                }
            }

            match self.current_screen.update() {
                ScreenAction::Quit => self.exit = true,
                ScreenAction::ChangeScreen(screen) => self.current_screen = screen,
                _ => {}
            }
        }
        Ok(())
    }
}
