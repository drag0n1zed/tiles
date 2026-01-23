mod grid;

use std::{
    error::Error,
    fs::{read_to_string, write},
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Widget},
};

use crate::grid::{Grid, TileMoveDirection};

fn main() -> Result<(), Box<dyn Error>> {
    // Import save file
    let grid = Grid::from_ron(read_to_string("grid.ron")?.as_str())?;

    let grid = ratatui::run(|terminal| {
        let mut app = App::from_grid(grid);
        app.run(terminal)?;
        let result: Result<Grid, Box<dyn Error>> = Ok(app.into_grid());
        result
    })?;

    // Export save file
    if false {
        write("grid.ron", grid.to_ron())?;
    }

    Ok(())
}

struct App {
    grid: Grid,

    exit: bool,
}

impl App {
    fn from_grid(grid: Grid) -> Self {
        Self { grid, exit: false }
    }

    fn into_grid(self) -> Grid {
        self.grid
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn Error>> {
        let tick_rate = Duration::from_millis(16);
        let mut last_tick = Instant::now();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)? {
                match event::read()? {
                    Event::Key(key) => self.handle_key_press(key),
                    _ => (),
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_press(&mut self, key: event::KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Left, _) => self.grid.move_grid(TileMoveDirection::Left),
            (KeyCode::Right, _) => self.grid.move_grid(TileMoveDirection::Right),
            (KeyCode::Up, _) => self.grid.move_grid(TileMoveDirection::Up),
            (KeyCode::Down, _) => self.grid.move_grid(TileMoveDirection::Down),
            (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => self.exit = true,
            _ => {}
        };
        self.grid.clear_connected_tiles();
    }
}

impl Widget for &App {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        let header = Line::from(" TILES ".bold());

        let footer = Line::from(" STEPS LEFT: to be implemented ").centered();

        let block = Block::bordered()
            .title(header)
            .border_set(border::THICK)
            .title_bottom(footer);

        let inner_rect = block.inner(rect);
        let inner_rect_with_margin = Rect::new(
            inner_rect.x + 2,
            inner_rect.y + 1,
            inner_rect.width - 2,
            inner_rect.height - 1,
        );

        block.render(rect, buf);

        self.grid.render(inner_rect_with_margin, buf);
    }
}
