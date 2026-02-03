use std::fs;

use ratatui::{
    Frame,
    buffer::Buffer,
    crossterm::event::{Event, KeyCode},
    layout::{Alignment, Margin, Position, Rect},
    style::{Color, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Clear, Paragraph, Widget},
};

use crate::{
    game::logic::grid::Grid,
    screens::{Screen, ScreenAction, game_screen::GameScreen},
};

pub struct MenuScreen;

impl MenuScreen {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for MenuScreen {
    fn handle_input(&mut self, event: Event) -> ScreenAction {
        let Event::Key(key) = event else {
            return ScreenAction::Nothing;
        };
        match key.code {
            KeyCode::Char('l') | KeyCode::Char('L') => {
                // TODO: open file picker and pick a .ron file
                ScreenAction::Nothing
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                // TODO: resume last played game
                let grid = Grid::from_ron(fs::read_to_string("grid.ron").unwrap().as_str()).unwrap();
                ScreenAction::ChangeScreen(Box::new(GameScreen::from_grid(grid)))
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                // TODO: 20-level challenge mode
                ScreenAction::Nothing
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                // TODO: Editor mode
                ScreenAction::Nothing
            }
            _ => ScreenAction::Nothing,
        }
    }

    fn render_screen(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn update(&mut self) -> ScreenAction {
        ScreenAction::Nothing
    }
}

impl Widget for &MenuScreen {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        let block = Block::bordered().border_set(border::THICK);
        let rect = block.inner(rect);

        // Dither
        let width = rect.width;
        let dither_width = (width as f32 * 0.60) as u16;
        let gradient_area = Rect {
            x: rect.right().saturating_sub(dither_width),
            y: rect.y,
            width: dither_width,
            height: rect.height,
        };

        let get_noise = |vx: u16, vy: u16, width: f32, left_offset: u16| -> bool {
            let relative_x = (vx.saturating_sub(left_offset)) as f32;
            let ratio = relative_x / width;
            let seed = (vx as u32).wrapping_mul(0x1E35A7BD) ^ (vy as u32).wrapping_mul(0x10E1);
            let mut hash = seed;
            hash = (hash ^ 61) ^ (hash >> 16);
            hash = hash.wrapping_add(hash << 3);
            hash = hash ^ (hash >> 4);
            hash = hash.wrapping_mul(0x27d4eb2d);
            hash = hash ^ (hash >> 15);
            let noise_val = (hash % 100) as f32 / 100.0;
            noise_val < ratio.powf(1.5)
        };

        let g_width = gradient_area.width as f32;
        let g_left = gradient_area.left();

        for x in gradient_area.left()..gradient_area.right() {
            for y in gradient_area.top()..gradient_area.bottom() {
                let top_active = get_noise(x, y * 2, g_width, g_left);
                let bot_active = get_noise(x, y * 2 + 1, g_width, g_left);

                if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                    cell.set_fg(Color::DarkGray);
                    cell.set_bg(Color::Reset);
                    let symbol = match (top_active, bot_active) {
                        (true, true) => "█",
                        (true, false) => "▀",
                        (false, true) => "▄",
                        (false, false) => " ",
                    };
                    cell.set_symbol(symbol);
                }
            }
        }

        let title_text = vec![
            "▀▀██▀▀ ▀▀██▀▀ ▀██▀   ▀██▀▀▀ ▄█▀▀█▄",
            "  ██     ██    ██     ██▄▄  ██▄▄  ",
            "  ██     ██    ██     ██▀▀    ▀▀██",
            " ▗██▖  ▄▄██▄▄ ▄██▄▄█ ▄██▄▄▄ ▀█▄▄█▀",
        ];
        let options = vec![
            Line::from(vec![
                Span::styled("(L)", Style::default().fg(Color::Yellow).bold()),
                Span::raw("oad Level"),
            ]),
            Line::from(vec![
                Span::styled("(R)", Style::default().fg(Color::Yellow).bold()),
                Span::raw("esume"),
            ]),
            Line::from(vec![
                Span::styled("(C)", Style::default().fg(Color::Yellow).bold()),
                Span::raw("hallenge Mode"),
            ]),
            Line::from(vec![
                Span::styled("(E)", Style::default().fg(Color::Yellow).bold()),
                Span::raw("ditor"),
            ]),
        ];

        let t_height = title_text.len() as u16;
        let t_width = title_text[0].chars().count() as u16;
        let m_height = options.len() as u16;
        let m_width = 16;

        let menu_rect = Rect {
            x: rect.x.saturating_add((rect.width as f64 * 0.0625) as u16),
            y: rect
                .bottom()
                .saturating_sub(m_height)
                .saturating_sub((rect.height as f64 * 0.1) as u16),
            width: m_width,
            height: m_height,
        };

        let title_rect = Rect {
            x: menu_rect.x,
            y: menu_rect
                .y
                .saturating_sub(t_height)
                .saturating_sub(((rect.height as f64 * 0.04) as u16).max(1)),
            width: t_width,
            height: t_height,
        };
        let union_rect = menu_rect.union(title_rect).outer(Margin::new(3, 2)).intersection(rect);
        Clear.render(union_rect, buf);
        Block::bordered().border_set(border::PLAIN).render(union_rect, buf);

        // draw title
        let title_paragraph = Paragraph::new(Text::from(title_text.join("\n")))
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::White).bold());
        title_paragraph.render(title_rect, buf);

        // draw menu
        let menu_paragraph = Paragraph::new(options).alignment(Alignment::Left);
        menu_paragraph.render(menu_rect, buf);
    }
}
