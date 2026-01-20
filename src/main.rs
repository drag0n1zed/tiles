mod objects;

use std::{
    error::Error,
    fs::{read_to_string, write},
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Padding, Paragraph, Widget},
};

use crate::objects::{Direction, Grid};

fn main() -> Result<(), Box<dyn Error>> {
    // Import save file
    let save_file = read_to_string("grid.ron")?;
    let grid = Grid::from_ron(save_file.as_str())?;

    let grid = ratatui::run(|terminal| {
        let mut app = App::from_grid(grid);
        app.run(terminal)?;
        let result: Result<Grid, Box<dyn Error>> = Ok(app.into_grid());
        result
    })?;

    // Export save file
    // let save_file = grid.to_ron();
    // println!("{}", save_file);
    // write("grid.ron", save_file)?;

    Ok(())
}

struct App {
    grid: Grid,
    exit: bool,
}

impl App {
    fn new(width: usize, length: usize) -> Self {
        Self {
            grid: Grid::new(width, length),
            exit: false,
        }
    }

    fn from_grid(grid: Grid) -> Self {
        Self { grid, exit: false }
    }

    fn into_grid(self) -> Grid {
        self.grid
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn Error>> {
        let tick_rate = Duration::from_millis(100);
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
        match key.code {
            KeyCode::Left => self.grid.move_grid(Direction::Left),
            KeyCode::Right => self.grid.move_grid(Direction::Right),
            KeyCode::Up => self.grid.move_grid(Direction::Up),
            KeyCode::Down => self.grid.move_grid(Direction::Down),
            KeyCode::Esc => self.exit = true,
            _ => {}
        }
    }
}

impl Widget for &App {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        let title = Line::from(" TILES ".bold());

        let padding = Padding::new(
            0,
            0,
            (rect.height - self.grid.get_length() as u16 - 1) / 2,
            0,
        );

        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK)
            .padding(padding);

        let text = Text::from(
            self.grid
                .to_vec()
                .pattern
                .iter()
                .map(|row| Line::from_iter(row))
                .collect::<Vec<_>>(),
        );

        Paragraph::new(text.centered())
            .centered()
            .block(block)
            .render(rect, buf);
    }
}
