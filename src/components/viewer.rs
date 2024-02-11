use std::collections::{BTreeMap, BTreeSet};

use crossterm::event::{Event, KeyCode, MouseEventKind};
use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Padding, Paragraph, Scrollbar, ScrollbarState},
    Frame,
};

use crate::{
    logger::Logger,
    utils::element_type_to_color,
    value::writer::Element,
    vi::{
        vimotions,
        vistate::{ViCommand, ViState},
    },
};

pub struct Viewer {
    logger: Logger,
    scroll: i32,
    curosr: [i32; 2],
    lines: Vec<Vec<Element>>,
    vistate: ViState,
    selections: BTreeMap<i32, BTreeSet<i32>>,
}

impl Viewer {
    pub fn new(logger: Logger) -> Self {
        Self {
            logger,
            scroll: 0,
            curosr: [0, 0],
            lines: Vec::new(),
            vistate: ViState::new(),
            selections: BTreeMap::new(),
        }
    }

    pub fn set_value_elemnets(&mut self, value_elements: Vec<Vec<Element>>) {
        self.lines = value_elements;
    }

    pub fn draw(&mut self, f: &mut Frame<'_>, chunk: Rect) {
        let container_h = (chunk.height as i32) - 2;

        self.curosr[0] = self.curosr[0].max(0).min(self.lines.len() as i32 - 1);

        let current_line_len = self.lines[self.curosr[0] as usize]
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

        self.scroll = self.scroll.min(self.lines.len() as i32 - 1).max(0);

        let line_start = self.scroll;
        let line_end = (self.scroll + container_h).min(self.lines.len() as i32);

        let lines = &self.lines[(line_start as usize)..(line_end as usize)];
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
                            let color = element_type_to_color(e.ty);
                            vec![
                                Span::from(&s[..c]).style(Style::default().fg(color)),
                                Span::from(&s[c..c + 1])
                                    .style(Style::default().fg(Color::Black).bg(color)),
                                ratatui::text::Span::from(&s[(c + 1)..])
                                    .style(Style::default().fg(color)),
                            ]
                        } else {
                            vec![Span::from(e.content.as_str())
                                .style(Style::default().fg(element_type_to_color(e.ty)))]
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
            ScrollbarState::new((self.lines.len() - (container_h as usize)).max(0))
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

    pub fn reset_input_state(&mut self) {
        self.vistate.reset();
    }

    fn current_line_char_vec(&self) -> Vec<char> {
        self.lines[self.curosr[0] as usize]
            .iter()
            .flat_map(|e| e.content.chars())
            .collect::<Vec<_>>()
    }

    fn move_curosr(&mut self, d_row: i32, d_col: i32) {
        self.curosr[0] += d_row;
        self.curosr[1] += d_col;
    }

    fn forward_word(&mut self) {
        let line = self.current_line_char_vec();
        let s = &line[self.curosr[1] as usize..];
        let delta = vimotions::next_word(s);
        self.curosr[1] += delta[1];
        if delta[0] > 0 {
            self.curosr[0] += delta[0];
            self.curosr[1] = 0; // TODO: this is not correct
        }
    }

    fn backward_word(&mut self) {
        let line = self.current_line_char_vec();
        let s = line[..=self.curosr[1] as usize]
            .iter()
            .copied()
            .rev()
            .collect::<Vec<_>>();
        let delta = vimotions::next_word(&s);
        self.curosr[1] -= delta[1];
        if delta[0] > 0 {
            self.curosr[0] -= delta[0];
            self.curosr[1] = 0; // TODO: this is not correct
        }
    }

    fn jump_next_char(&mut self, c: char) {
        let line = self.current_line_char_vec();
        if line.is_empty() {
            self.curosr[1] = 0;
            return;
        }
        let i = (self.curosr[1] as usize).min(line.len() - 1);
        let line = &line[i..];
        let delta = vimotions::jump_next_char(&line, c, self.logger.clone());
        self.curosr[1] += delta;
    }

    fn jump_prev_char(&mut self, c: char) {
        let line = self.current_line_char_vec();
        if line.is_empty() {
            self.curosr[1] = 0;
            return;
        }
        let i = (self.curosr[1] as usize).min(line.len() - 1);
        let line = line[..=i].iter().rev().copied().collect::<Vec<_>>();
        let delta = vimotions::jump_next_char(line.as_slice(), c, self.logger.clone());
        self.curosr[1] -= delta;
    }

    fn first_line(&mut self) {
        self.curosr[0] = 0;
    }

    fn last_line(&mut self) {
        self.curosr[0] = (self.lines.len() - 1) as _;
    }

    fn first_column(&mut self) {
        self.curosr[1] = 0;
    }

    fn last_column(&mut self) {
        let line_len = self.current_line_char_vec().len();
        self.curosr[1] = (line_len - 1).max(0) as i32;
    }

    fn next(&mut self) {
        if let Some((&row, selections)) = self.selections.range(self.curosr[0]..).next() {
            let range_start = if self.curosr[0] == row {
                self.curosr[1]
            } else {
                0
            };

            if let Some(&col) = selections.range(range_start..).next() {
                self.curosr[0] = row;
                self.curosr[1] = col;
            }
        }
    }

    fn previous(&mut self) {
        todo!()
    }

    fn process_command(&mut self, command: ViCommand) {
        type C = ViCommand;
        match command {
            // simple navigation
            C::Up => self.move_curosr(-1, 0),
            C::Down => self.move_curosr(1, 0),
            C::Left => self.move_curosr(0, -1),
            C::Right => self.move_curosr(0, 1),
            C::MoveWordForward => self.forward_word(),
            C::MoveWordBackward => self.backward_word(),
            // jumps
            C::JumpNextChar(c) => self.jump_next_char(c),
            C::JumpPreviousChar(c) => self.jump_prev_char(c),
            // global movement
            C::FirstLine => self.first_line(),
            C::LastLine => self.last_line(),
            C::FirstColumn => self.first_column(),
            C::LastColumn => self.last_column(),
            // search results
            C::Next => self.next(),
            C::Previous => self.previous(),
        }
    }

    pub fn handle_input(&mut self, event: Option<Event>) {
        match event {
            Some(Event::Key(ke)) => match ke.code {
                KeyCode::Char(c) => {
                    if let Some(command) = self.vistate.process(c) {
                        self.process_command(command);
                    }
                }
                _ => {}
            },
            Some(Event::Mouse(me)) => match me.kind {
                MouseEventKind::ScrollUp => self.process_command(ViCommand::Up),
                MouseEventKind::ScrollDown => self.process_command(ViCommand::Down),
                MouseEventKind::ScrollLeft => self.process_command(ViCommand::Left),
                MouseEventKind::ScrollRight => self.process_command(ViCommand::Right),
                _ => {}
            },
            _ => {}
        }
    }
}
