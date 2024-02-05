use derive_more::IsVariant;
use std::{default, path::Path, pin::Pin};

use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Scrollbar, ScrollbarState},
    Frame,
};

use crate::{
    utils::{self, centered_rect, container_block, inset_chunk},
    value::{writer::Element, ContainedValue},
};

#[derive(Debug, IsVariant, Clone, Copy, PartialEq, Eq)]
pub enum CurrentScreen {
    Viewer,
    Query,
}

impl CurrentScreen {
    fn toggle(self) -> Self {
        match self {
            Self::Query => Self::Viewer,
            Self::Viewer => Self::Query,
        }
    }
}

pub struct App<'a> {
    done: bool,
    value: Pin<Box<ContainedValue<'a>>>,
    query: String,
    current_screen: CurrentScreen,
    scroll: i32,
    curosr: [i32; 2],
    value_elements: Vec<Vec<Element>>,
    show_logs: bool,
    logs: Vec<String>,
}

impl<'a> App<'a> {
    pub fn new(file: impl AsRef<Path>) -> Self {
        let source = std::fs::read_to_string(file).unwrap();
        let mut out = Self {
            value: ContainedValue::parse(source),
            done: false,
            query: String::new(),
            current_screen: CurrentScreen::Query,
            scroll: 0,
            curosr: [0, 0],
            value_elements: Vec::new(),
            show_logs: false,
            logs: Vec::new(),
        };
        out.value_elements = out.value.get().elemnets();
        out
    }

    fn log(&mut self, log: impl ToString) {
        self.logs.push(log.to_string());
    }

    pub fn done(&self) -> bool {
        self.done
    }

    fn add_to_query(&mut self, c: char) {
        self.query = format!("{}{}", self.query, c);
        self.recalculate_query();
    }

    fn delete_query_char(&mut self) {
        if !self.query.is_empty() {
            self.query = self.query[..self.query.len() - 1].to_string();
            self.recalculate_query();
        }
    }

    fn recalculate_query(&mut self) {
        // TODO
        self.log("recalculate_query");
    }

    fn toggle_screen(&mut self) {
        self.current_screen = self.current_screen.toggle();
        self.log(format!("{:?}", self.current_screen));
    }

    pub fn draw(&mut self, f: &mut Frame<'_>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Length(30)])
            .split(f.size());
        self.render_viewer_screen(f, chunks[0], self.current_screen.is_viewer());
        self.render_query_screen(f, chunks[1], self.current_screen.is_query());

        if self.show_logs {
            self.render_logs(f);
        }
    }

    fn render_logs(&self, f: &mut Frame<'_>) {
        let block = Block::default()
            .title("Logs")
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .style(Style::default().bg(Color::DarkGray));
        let p = Paragraph::new(
            self.logs
                .clone()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .block(block);

        f.render_widget(p, centered_rect(60, 25, f.size()));
    }

    fn render_viewer_screen(&mut self, f: &mut Frame<'_>, chunk: Rect, active: bool) {
        let chunk = inset_chunk(chunk);

        //outer
        f.render_widget(container_block("Explorer", active), chunk);

        let container_h = (chunk.height as i32) - 2;

        self.curosr[0] = self.curosr[0]
            .max(0)
            .min(self.value_elements.len() as i32 - 1);

        let current_line_len = self.value_elements[self.curosr[0] as usize]
            .iter()
            .map(|e| e.content.len() as i32)
            .sum::<i32>();

        self.curosr[1] = self.curosr[1].max(0).min(current_line_len - 1);

        if self.curosr[0] < self.scroll {
            self.scroll = self.curosr[0]
        };

        while self.scroll + container_h <= self.curosr[0] {
            self.scroll += 1;
        }

        self.scroll = self.scroll.min(self.value_elements.len() as i32 - 1).max(0);

        let line_start = self.scroll;
        let line_end = (self.scroll + container_h).min(self.value_elements.len() as i32);

        let lines = &self.value_elements[(line_start as usize)..(line_end as usize)];
        let lines = lines
            .into_iter()
            .enumerate()
            .map(|(i, line)| {
                let mut start_col = 0i32;
                let spans = line
                    .iter()
                    .flat_map(|e| {
                        let i = i as i32 + line_start;
                        let out = if i == self.curosr[0]
                            && self.curosr[1] >= start_col
                            && self.curosr[1] < start_col + e.content.len() as i32
                        {
                            let s = e.content.as_str();
                            let c = (self.curosr[1] - start_col) as usize;
                            let color = utils::element_type_to_color(e.ty);
                            vec![
                                ratatui::text::Span::from(&s[..c])
                                    .style(Style::default().fg(color)),
                                ratatui::text::Span::from(&s[c..c + 1])
                                    .style(Style::default().fg(Color::Black).bg(color)),
                                ratatui::text::Span::from(&s[(c + 1)..])
                                    .style(Style::default().fg(color)),
                            ]
                        } else {
                            vec![ratatui::text::Span::from(e.content.as_str())
                                .style(Style::default().fg(utils::element_type_to_color(e.ty)))]
                        };

                        start_col += e.content.len() as i32;
                        out
                    })
                    .collect::<Vec<_>>();
                ratatui::text::Line::from(spans)
            })
            .collect::<Vec<_>>();

        f.render_widget(
            Paragraph::new(lines)
                .block(Block::default().padding(Padding::new(1, 1, 1, 1)))
                .scroll((0, 0)),
            chunk,
        );

        let mut scrollbar_state =
            ScrollbarState::new((self.value_elements.len() - (container_h as usize)).max(0))
                .position(self.scroll as _);
        f.render_stateful_widget(
            Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight),
            chunk.inner(&Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }

    fn render_query_screen(&mut self, f: &mut Frame<'_>, chunk: Rect, active: bool) {
        let chunk = inset_chunk(chunk);

        // outer
        f.render_widget(container_block("Query", active), chunk);

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

    pub fn handle_event(&mut self, event: Option<Event>) {
        match self.current_screen {
            CurrentScreen::Viewer => match event {
                Some(Event::Key(ke)) => match ke.code {
                    KeyCode::Char('q') => self.done = true,
                    KeyCode::Tab => self.toggle_screen(),
                    KeyCode::Char('I') => self.show_logs = !self.show_logs,
                    KeyCode::Char('h') | KeyCode::Left => self.curosr[1] -= 1,
                    KeyCode::Char('j') | KeyCode::Down => self.curosr[0] += 1,
                    KeyCode::Char('k') | KeyCode::Up => self.curosr[0] -= 1,
                    KeyCode::Char('l') | KeyCode::Right => self.curosr[1] += 1,
                    _ => {}
                },
                _ => {}
            },
            CurrentScreen::Query => match event {
                Some(Event::Key(ke)) => match ke.code {
                    KeyCode::Tab => self.toggle_screen(),
                    KeyCode::Char(c) => {
                        self.add_to_query(c);
                    }
                    KeyCode::Backspace => self.delete_query_char(),
                    _ => {}
                },
                _ => {}
            },
        }
    }
}
