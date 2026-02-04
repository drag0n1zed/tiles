use std::{
    cell::RefCell,
    env, fs,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{Context, Result};
use ratatui::{
    Frame,
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget},
};

use crate::{
    game::{logic::grid::Grid, ui::grid_widget::GridWidget},
    screens::{Screen, ScreenAction, game::GameScreen},
};

pub struct FileItem {
    name: String,
    path: PathBuf,
    is_dir: bool,
}

pub struct FilePickerScreen {
    current_dir: PathBuf,
    items: Vec<FileItem>,
    state: RefCell<ListState>,
}

impl FilePickerScreen {
    pub fn new() -> Self {
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut screen = Self {
            current_dir,
            items: Vec::new(),
            state: RefCell::new(ListState::default()),
        };
        screen.refresh_items();
        if !screen.items.is_empty() {
            screen.state.borrow_mut().select(Some(0));
        }
        screen
    }

    fn get_items_in_dir(path: &Path) -> Vec<FileItem> {
        let mut items = Vec::new();
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let is_dir = path.is_dir();
                let name = entry.file_name().to_string_lossy().to_string() + (if is_dir { "/" } else { "" });

                if is_dir | name.ends_with(".ron") {
                    items.push(FileItem { name, path, is_dir });
                }
            }
        }
        items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });
        items
    }

    fn refresh_items(&mut self) {
        self.items = Self::get_items_in_dir(&self.current_dir);
        self.state.borrow_mut().select(Some(0));
    }

    fn go_up(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = self.state.borrow().selected().unwrap_or(0);
        if i > 0 {
            self.state.borrow_mut().select(Some(i - 1));
        } else {
            self.state.borrow_mut().select(Some(self.items.len() - 1));
        }
    }

    fn go_down(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = self.state.borrow().selected().unwrap_or(0);
        if i < self.items.len() - 1 {
            self.state.borrow_mut().select(Some(i + 1));
        } else {
            self.state.borrow_mut().select(Some(0));
        }
    }

    fn go_left(&mut self) {
        match self.current_dir.parent() {
            Some(parent) => {
                let child_dir = self.current_dir.clone();
                self.current_dir = parent.to_path_buf();
                self.refresh_items();
                self.state
                    .borrow_mut()
                    .select(self.items.iter().position(|i| i.path == child_dir));
            }
            None => {}
        }
    }

    fn select_current(&mut self) -> Result<ScreenAction> {
        let Some(index) = self.state.borrow().selected() else {
            // Nothing selected
            return Ok(ScreenAction::Nothing);
        };
        if let Some(FileItem { path, is_dir, .. }) = self.items.get(index) {
            if *is_dir {
                self.current_dir = path.clone();
                self.refresh_items();
            } else {
                return Self::get_action_from_grid(Self::load_grid_from_path(path));
            }
        }
        Ok(ScreenAction::Nothing)
    }

    fn get_action_from_grid(grid: Result<Grid>) -> Result<ScreenAction> {
        Ok(GameScreen::from_grid(grid?).into())
    }

    fn load_grid_from_path(path: &Path) -> Result<Grid> {
        let content = fs::read_to_string(&path).wrap_err("Could not read file")?;
        Ok(Grid::from_ron(&content).wrap_err("Invalid RON format")?)
    }

    fn render_file_list(
        items: &[FileItem],
        rect: Rect,
        buf: &mut Buffer,
        selected_index: Option<usize>,
        right_border: bool,
        is_active: bool,
    ) {
        let list_items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let color = match (is_active, selected_index == Some(i), item.is_dir) {
                    (true, true, true) => Color::Blue,
                    (true, true, false) => Color::Red,
                    (true, false, _) => Color::White,
                    _ => Color::DarkGray,
                };

                let mut style = Style::default().fg(color);
                if !item.is_dir {
                    style = style.underlined().italic();
                }

                ListItem::new(Line::from(Span::styled(&item.name, style)))
            })
            .collect();

        let block = Block::bordered()
            .borders(if right_border { Borders::RIGHT } else { Borders::empty() })
            .border_style(Style::default().fg(Color::DarkGray))
            .border_set(if is_active { border::THICK } else { border::PLAIN });

        let mut state = ListState::default();
        state.select(selected_index);
        StatefulWidget::render(
            List::new(list_items).block(block).highlight_symbol("> "),
            rect,
            buf,
            &mut state,
        );
    }
}

impl Screen for FilePickerScreen {
    fn update(&mut self, event: Option<KeyEvent>) -> Result<ScreenAction> {
        if let Some(key) = event {
            match key.code {
                KeyCode::Left => self.go_left(),
                KeyCode::Up => self.go_up(),
                KeyCode::Down => self.go_down(),
                KeyCode::Right | KeyCode::Enter => return self.select_current(),
                _ => {}
            }
        }
        Ok(ScreenAction::Nothing)
    }
    fn render_screen(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &FilePickerScreen {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        let big_block = Block::bordered()
            .border_set(border::PLAIN)
            .title(Line::from(" TILES ".bold()))
            .title_bottom(Line::from(format!(" {} ", self.current_dir.display())));

        let inner_rect = big_block.inner(rect);
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(40),
                Constraint::Percentage(40),
            ])
            .split(inner_rect);
        let parent_rect = chunks[0];
        let current_rect = chunks[1];
        let preview_rect = chunks[2];

        big_block.render(rect, buf);

        if let Some(parent_path) = self.current_dir.parent() {
            let parent_items = FilePickerScreen::get_items_in_dir(parent_path);
            let selected_idx = parent_items.iter().position(|i| i.path == self.current_dir);
            FilePickerScreen::render_file_list(&parent_items, parent_rect, buf, selected_idx, true, false);
        } else {
            Block::bordered().border_set(border::PLAIN).render(parent_rect, buf);
        }

        FilePickerScreen::render_file_list(
            &self.items,
            current_rect,
            buf,
            self.state.borrow().selected(),
            true,
            true,
        );

        if let Some(idx) = self.state.borrow().selected() {
            if let Some(item) = self.items.get(idx) {
                match item.is_dir {
                    true => {
                        let child_items = FilePickerScreen::get_items_in_dir(&item.path);
                        FilePickerScreen::render_file_list(&child_items, preview_rect, buf, None, false, false);
                    }
                    false => {
                        match FilePickerScreen::load_grid_from_path(&item.path) {
                            Ok(grid) => GridWidget::new(&grid).render(preview_rect, buf),
                            Err(r) => {}
                        };
                    }
                }
            }
        }
    }
}
