use ratatui::{
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::{logger::Logger, utils::centered_rect};

pub struct LogPopup {
    logger: Logger,
}

impl LogPopup {
    pub fn new(logger: Logger) -> Self {
        Self { logger }
    }

    // TODO: figure out why the popup doesn't override its background
    pub fn draw(&self, f: &mut Frame<'_>) {
        let block = Block::default()
            .title("Logs")
            .bg(Color::DarkGray)
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .style(Style::default().bg(Color::DarkGray));
        let p = Paragraph::new(
            self.logger
                .get_all_logs()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .block(block.clone());

        let area = centered_rect(60, 25, f.size());
        f.render_widget(block, area);
        f.render_widget(p, area);
    }
}
