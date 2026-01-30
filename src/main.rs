mod grid;
mod timer;

use std::{collections::VecDeque, fs, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Styled, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Widget},
};

use crate::{
    grid::{Grid, MoveDirection},
    timer::Timer,
};

fn main() -> Result<()> {
    color_eyre::install()?;
    // Import save file
    let grid = Grid::from_ron(fs::read_to_string("grid.ron")?.as_str())?;

    let grid = ratatui::run(|terminal| {
        let mut app = App::from_grid(grid);
        app.run(terminal)?;
        let result: Result<Grid> = Ok(app.into_grid());
        result
    })?;

    // Export save file
    if false {
        fs::write("grid.ron", grid.to_ron())?;
    }

    Ok(())
}

struct App {
    grid: Grid,
    input_queue: VecDeque<MoveDirection>,
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
        let mut game_tick_timer = Timer::new(Duration::from_millis(8));
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(game_tick_timer.time_until_ready())? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_press(key);
                }
            }

            // Execute every tick:
            if game_tick_timer.ready() {
                self.grid.update_anim_state();

                if self.grid.is_anim_completed()
                    && let Some(input) = self.input_queue.pop_front()
                {
                    self.grid.move_grid(input);
                }
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
            let input: Option<MoveDirection> = match (key.code, key.modifiers) {
                (KeyCode::Left, _) => Some(MoveDirection::Left),
                (KeyCode::Right, _) => Some(MoveDirection::Right),
                (KeyCode::Up, _) => Some(MoveDirection::Up),
                (KeyCode::Down, _) => Some(MoveDirection::Down),
                _ => None,
            };
            self.input_queue.extend(input);
        }
    }
}

impl Widget for &App {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        let header = Line::from(" TILES ".bold());

        let footer = {
            let spans = vec![
                Span::raw(" "),
                Span::raw(self.grid.steps.to_string()).bold(),
                Span::raw(" moves remaining "),
            ];

            Line::from(spans)
        };

        let block = Block::bordered()
            .title(header)
            .border_set(border::THICK)
            .title_bottom(footer.centered())
            .set_style(Style::default().bg(Color::Reset));

        let inner_rect = block.inner(rect);
        let inner_rect_with_margin = Rect::new(
            inner_rect.x + 2,
            inner_rect.y + 1,
            inner_rect.width - 2,
            inner_rect.height - 1,
        );

        block.render(rect, buf);

        self.grid.as_widget().render(inner_rect_with_margin, buf);
    }
}
