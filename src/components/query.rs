use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::logger::Logger;

pub struct Query {
    logger: Logger,
    query: String,
    query_changed: bool,
}

impl Query {
    pub fn new(logger: Logger) -> Self {
        Self {
            logger,
            query: String::new(),
            query_changed: false,
        }
    }

    pub fn get_if_changed(&mut self) -> Option<&str> {
        if self.query_changed {
            self.query_changed = false;
            Some(&self.query)
        } else {
            None
        }
    }

    fn delete_query_char(&mut self) {
        if !self.query.is_empty() {
            self.query_changed = true;
            self.query = self.query[..self.query.len() - 1].to_string();
        }
    }

    fn add_to_query(&mut self, c: char) {
        self.query_changed = true;
        self.query = format!("{}{}", self.query, c);
    }

    pub fn reset_input_state(&mut self) {}

    pub fn handle_input(&mut self, event: Option<Event>) {
        match event {
            Some(Event::Key(ke)) => match ke.code {
                KeyCode::Char(c) => {
                    self.add_to_query(c);
                }
                KeyCode::Backspace => self.delete_query_char(),
                _ => {}
            },
            _ => {}
        }
    }

    pub fn draw(&mut self, f: &mut Frame<'_>, chunk: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(chunk);

        // query text area
        let query_text = if self.query.is_empty() {
            "enter query here..."
        } else {
            &self.query
        };
        let base = Paragraph::new(query_text)
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(
            base,
            // if self.query.is_empty() {
            //     base.gray()
            // } else {
            //     base.gray()
            // },
            chunks[0],
        );

        // f.render_widget(Paragraph::new("TODO: bottom").white(), chunks[1]);
    }
}
