use derive_more::IsVariant;
use std::{path::Path, pin::Pin, rc::Rc};

use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::{
    components::{log_popup::LogPopup, query::Query, viewer::Viewer},
    logger::Logger,
    utils::{container_block, inset_chunk},
    value::ContainedValue,
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
    current_screen: CurrentScreen,
    logger: Logger,
    show_logs: bool,
    logs: LogPopup,
    viewer: Viewer,
    query: Query,
}

impl<'a> App<'a> {
    pub fn new(file: impl AsRef<Path>) -> Self {
        let source = std::fs::read_to_string(file).unwrap();
        let logger = Logger::new();
        let mut out = Self {
            logger: logger.clone(),
            value: ContainedValue::parse(source),
            done: false,
            current_screen: CurrentScreen::Query,
            show_logs: false,
            logs: LogPopup::new(logger.clone()),
            viewer: Viewer::new(logger.clone()),
            query: Query::new(logger.clone()),
        };
        out.viewer.set_value_elemnets(out.value.get().elemnets());
        out
    }

    pub fn done(&self) -> bool {
        self.done
    }

    fn recalculate_query(&mut self, _query: &str) {
        self.logger.log("recalculate_query");
    }

    fn toggle_screen(&mut self) {
        self.current_screen = self.current_screen.toggle();
    }

    pub fn draw(&mut self, f: &mut Frame<'_>) {
        if let Some(new_query) = self.query.get_if_changed().map(|s| s.to_string()) {
            self.recalculate_query(&new_query);
        }

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Length(30)])
            .split(f.size());
        self.render_viewer_screen(f, chunks[0], self.current_screen.is_viewer());
        self.render_query_screen(f, chunks[1], self.current_screen.is_query());

        if self.show_logs {
            self.logs.draw(f);
        }
    }

    fn render_viewer_screen(&mut self, f: &mut Frame<'_>, chunk: Rect, active: bool) {
        let chunk = inset_chunk(chunk);
        f.render_widget(container_block("Explorer", active), chunk);
        self.viewer.draw(f, chunk);
    }

    fn render_query_screen(&mut self, f: &mut Frame<'_>, chunk: Rect, active: bool) {
        let chunk = inset_chunk(chunk);
        f.render_widget(container_block("Query", active), chunk);
        self.query.draw(f, chunk);
    }

    pub fn handle_event(&mut self, event: Option<Event>) {
        match event {
            Some(Event::Key(ke)) => match ke.code {
                KeyCode::Char('q') => self.done = true,
                KeyCode::Tab => self.toggle_screen(),
                KeyCode::Char('I') => self.show_logs = !self.show_logs,
                _ => {}
            },
            _ => {}
        }

        match self.current_screen {
            CurrentScreen::Viewer => {
                self.viewer.handle_input(event);
                self.query.reset_input_state();
            }
            CurrentScreen::Query => {
                self.viewer.reset_input_state();
                self.query.handle_input(event);
            }
        }
    }
}
