use std::collections::VecDeque;

use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::Block;

use crate::{
    game::{
        logic::grid::{Grid, MoveDir},
        ui::grid_widget::GridWidget,
    },
    screens::{Screen, ScreenAction},
};

pub struct GameScreen {
    grid: Grid,
    input_queue: VecDeque<MoveDir>,
}

impl GameScreen {
    pub fn from_grid(grid: Grid) -> Self {
        GameScreen {
            grid,
            input_queue: VecDeque::new(),
        }
    }
}

impl Screen for GameScreen {
    fn handle_input(&mut self, event: Event) -> ScreenAction {
        let Event::Key(key) = event else {
            return ScreenAction::Nothing;
        };
        if self.input_queue.len() <= 2 {
            let input: Option<MoveDir> = match (key.code, key.modifiers) {
                (KeyCode::Left, _) => Some(MoveDir::Left),
                (KeyCode::Right, _) => Some(MoveDir::Right),
                (KeyCode::Up, _) => Some(MoveDir::Up),
                (KeyCode::Down, _) => Some(MoveDir::Down),
                _ => None,
            };
            self.input_queue.extend(input);
        };
        ScreenAction::Nothing
    }

    fn update(&mut self) -> ScreenAction {
        self.grid.update_anim_state();

        if self.grid.is_anim_completed()
            && let Some(input) = self.input_queue.pop_front()
        {
            self.grid.move_grid(input);
        }
        // TODO: Return ChangeState if all cleared. "Success!"

        ScreenAction::Nothing
    }

    fn render_screen(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &GameScreen {
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
            .title_bottom(footer.centered());

        let inner_rect = block.inner(rect);

        block.render(rect, buf);

        GridWidget::new(&self.grid).render(inner_rect, buf);
    }
}
