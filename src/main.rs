mod grid;

use std::{
    collections::VecDeque,
    fs::{read_to_string, write},
    time::{Duration, Instant},
};

use color_eyre::eyre::Result;
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

fn main() -> Result<()> {
    color_eyre::install()?;
    // Import save file
    let grid = Grid::from_ron(read_to_string("grid.ron")?.as_str())?;

    let grid = ratatui::run(|terminal| {
        let mut app = App::from_grid(grid);
        app.run(terminal)?;
        let result: Result<Grid> = Ok(app.into_grid());
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
    input_queue: VecDeque<TileMoveDirection>,
    exit: bool,
}

impl App {
    fn from_grid(grid: Grid) -> Self {
        Self {
            grid,
            input_queue: VecDeque::new(),
            exit: false,
        }
    }

    fn into_grid(self) -> Grid {
        self.grid
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let tick_rate = Duration::from_millis(8);
        let mut last_tick = Instant::now();
        let mut dirty = true;
        while !self.exit {
            if dirty {
                terminal.draw(|frame| self.draw(frame))?;
            }
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            let anim_ongoing = self.grid.update_grid();
            dirty |= anim_ongoing;

            if !anim_ongoing {
                if let Some(input) = self.input_queue.pop_front() {
                    self.grid.move_grid(input);
                }
            }

            if event::poll(timeout)? {
                match event::read()? {
                    Event::Key(key) => self.handle_key_press(key),
                    Event::Resize(_, _) => dirty |= true,
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
            (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => self.exit = true,
            _ => {}
        };
        if self.input_queue.len() <= 2 {
            let input: Option<TileMoveDirection> = match (key.code, key.modifiers) {
                (KeyCode::Left, _) => Some(TileMoveDirection::Left),
                (KeyCode::Right, _) => Some(TileMoveDirection::Right),
                (KeyCode::Up, _) => Some(TileMoveDirection::Up),
                (KeyCode::Down, _) => Some(TileMoveDirection::Down),
                _ => None,
            };
            self.input_queue.extend(input);
        }
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
