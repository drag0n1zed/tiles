use std::fs;

use ratatui::{
    Frame,
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Alignment, Margin, Position, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Clear, Paragraph, Widget},
};

use crate::{
    game::logic::grid::Grid,
    screens::{Screen, ScreenAction, game::GameScreen},
};

pub struct MenuOption<'a> {
    display: Line<'a>,
    action: Box<dyn Fn() -> ScreenAction + 'a>,
}

pub struct MenuScreen<'a> {
    options: Vec<MenuOption<'a>>,
    selected_index: usize,
}

impl<'a> MenuScreen<'a> {
    pub fn new(options: Vec<MenuOption<'a>>) -> Self {
        Self {
            options,
            selected_index: 0,
        }
    }

    pub fn main_menu() -> Self {
        Self::new(vec![
            MenuOption {
                display: Line::raw("Begin Challenge"),
                action: Box::new(|| {
                    // TODO: open file picker and pick a .ron file
                    ScreenAction::Nothing
                }),
            },
            MenuOption {
                display: Line::raw("Resume"),
                action: Box::new(|| {
                    // TODO: resume last played game
                    let grid = Grid::from_ron(fs::read_to_string("grid.ron").unwrap().as_str()).unwrap();
                    GameScreen::from_grid(grid).into()
                }),
            },
            MenuOption {
                display: Line::raw("Custom"),
                action: Box::new(|| MenuScreen::custom_menu().into()),
            },
            MenuOption {
                display: Line::raw("Quit"),
                action: Box::new(|| ScreenAction::PopScreen),
            },
        ])
    }

    pub fn custom_menu() -> Self {
        Self::new(vec![
            MenuOption {
                display: Line::raw("Open local game"),
                action: Box::new(|| {
                    // Start up file selection thing
                    ScreenAction::Nothing
                }),
            },
            MenuOption {
                display: Line::raw("Recent games"),
                action: Box::new(|| {
                    // give a list of recently opened games
                    ScreenAction::Nothing
                }),
            },
            MenuOption {
                display: Line::raw("Back"),
                action: Box::new(|| ScreenAction::PopScreen),
            },
        ])
    }
}

impl Screen for MenuScreen<'_> {
    fn render_screen(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn update(&mut self, event: Option<KeyEvent>) -> ScreenAction {
        if let Some(key) = event {
            match key.code {
                KeyCode::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    } else {
                        self.selected_index = self.options.len() - 1;
                    }
                }
                KeyCode::Down => {
                    if self.selected_index < self.options.len() - 1 {
                        self.selected_index += 1;
                    } else {
                        self.selected_index = 0;
                    }
                }
                KeyCode::Enter => return (self.options[self.selected_index].action)(),
                _ => {}
            }
        }
        ScreenAction::Nothing
    }
}

impl Widget for &MenuScreen<'_> {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        let block = Block::bordered().border_set(border::THICK);
        let rect = block.inner(rect);

        // Dither
        let width = rect.width;
        let dither_width = (width as f32 * 0.70) as u16;
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

        let title = vec![
            "▀▀██▀▀ ▀▀██▀▀ ▀██▀   ▀██▀▀▀ ▄█▀▀█▄",
            "  ██     ██    ██     ██▄▄  ██▄▄  ",
            "  ██     ██    ██     ██▀▀    ▀▀██",
            " ▗██▖  ▄▄██▄▄ ▄██▄▄█ ▄██▄▄▄ ▀█▄▄█▀",
        ];

        let t_height = title.len() as u16;
        let t_width = title.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;

        let m_height = self.options.len() as u16;
        // Width + 2 for the pointer "> "
        let m_width = self.options.iter().map(|o| o.display.width()).max().unwrap_or(0) as u16 + 2;

        let menu_rect = Rect {
            x: rect.x.saturating_add((rect.width as f64 * 0.08) as u16),
            y: rect
                .bottom()
                .saturating_sub(m_height)
                .saturating_sub((rect.height as f64 * 0.15) as u16),
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
        let title_paragraph = Paragraph::new(Text::from(title.join("\n")))
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::White).bold());
        title_paragraph.render(title_rect, buf);

        // draw menu
        let option_lines: Vec<Line> = self
            .options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let is_selected = i == self.selected_index;
                let mut spans = Vec::new();

                if is_selected {
                    // ARROW
                    spans.push(Span::styled("> ", Style::default().fg(Color::Yellow)));
                    for span in &option.display.spans {
                        spans.push(span.clone().fg(Color::Yellow));
                    }
                } else {
                    // NO ARROW
                    spans.push(Span::raw("  "));
                    spans.extend(option.display.spans.clone());
                }

                Line::from(spans)
            })
            .collect();

        let menu_paragraph = Paragraph::new(option_lines).alignment(Alignment::Left);
        menu_paragraph.render(menu_rect, buf);
    }
}
