use ratatui::{
    Frame,
    buffer::Buffer,
    crossterm::event::{Event, KeyCode},
    layout::{Alignment, Position, Rect},
    style::{Color, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::screens::{Screen, ScreenAction};

pub struct MenuScreen {
    pending_action: ScreenAction,
}

impl MenuScreen {
    pub fn new() -> Self {
        Self {
            pending_action: ScreenAction::Nothing,
        }
    }
}

impl Screen for MenuScreen {
    fn handle_input(&mut self, event: Event) {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    println!("Action: New Game");
                    // self.pending_action = ScreenAction::NewGame;
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    println!("Action: Resume");
                    // self.pending_action = ScreenAction::Resume;
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    println!("Action: Challenge Mode");
                    // self.pending_action = ScreenAction::Challenge;
                }
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    println!("Action: Editor");
                    // self.pending_action = ScreenAction::Editor;
                }
                _ => {}
            }
        }
    }

    fn render_screen(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn update(&mut self) -> ScreenAction {
        std::mem::take(&mut self.pending_action)
    }
}

impl Widget for &MenuScreen {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        let block = Block::bordered().border_set(border::THICK);
        let rect = block.inner(rect);

        // Border
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));
        let inner_area = block.inner(rect);
        block.render(rect, buf);

        // Dither
        let width = inner_area.width;
        let dither_width = (width as f32 * 0.60) as u16;
        let gradient_area = Rect {
            x: inner_area.right().saturating_sub(dither_width),
            y: inner_area.y,
            width: dither_width,
            height: inner_area.height,
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

        let half_phi = 0.191; // half of golden ratio
        let one_minus_half_phi = 0.809; // one minus half of golden ratio

        let center_x = inner_area.x + (inner_area.width as f32 * half_phi) as u16;
        let menu_center_y = inner_area.y + (inner_area.height as f32 * one_minus_half_phi) as u16;

        let title_text = vec![
            "████████ ██████  ██      ██████  ██████",
            "   ██      ██    ██      ██      ██    ",
            "   ██      ██    ██      █████   ██████",
            "   ██      ██    ██      ██          ██",
            "   ██    ██████  ██████  ██████  ██████",
        ];
        let options = vec![
            Line::from(vec![
                Span::styled("(N)", Style::default().fg(Color::Yellow).bold()),
                Span::raw("ew Game"),
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
        let m_width = 20;

        // menu
        let menu_area = Rect {
            x: center_x.saturating_sub(m_width / 2),
            y: menu_center_y.saturating_sub(m_height / 2),
            width: m_width,
            height: m_height,
        };

        // title above menu
        let padding = 3;
        let title_area = Rect {
            x: menu_area.x.saturating_sub(1),
            y: menu_area.y.saturating_sub(t_height + padding),
            width: t_width,
            height: t_height,
        };

        // clear background behind text helper thing
        let clear_background = |area: Rect, buf: &mut Buffer| {
            for y in area.top()..area.bottom() {
                for x in area.left()..area.right() {
                    if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                        cell.set_bg(Color::Reset);
                    }
                }
            }
        };

        // draw title
        clear_background(title_area, buf);
        let title_paragraph = Paragraph::new(Text::from(title_text.join("\n")))
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::White).bold());
        title_paragraph.render(title_area, buf);

        // draw menu
        clear_background(menu_area, buf);
        let menu_paragraph = Paragraph::new(options).alignment(Alignment::Left);
        menu_paragraph.render(menu_area, buf);
    }
}
