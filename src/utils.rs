use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, Padding},
};

use crate::value::writer::ElementType;

pub fn container_block<'b>(title: &'b str, active: bool) -> Block<'b> {
    let p = 0;
    if active {
        Block::default().border_type(BorderType::Thick)
    } else {
        Block::default().border_type(BorderType::Plain)
    }
    .borders(Borders::ALL)
    .title(Span::from(title).bold())
    .white()
    // .bold()
    .padding(Padding::new(p, p, p, p))
}

pub fn element_type_to_color(ty: ElementType) -> Color {
    match ty {
        ElementType::Whtiespace => Color::Gray,
        ElementType::Comma => Color::Gray,
        ElementType::Key => Color::Cyan,
        ElementType::NullLiteral => Color::LightYellow,
        ElementType::BoolLiteral => Color::LightYellow,
        ElementType::NumberLiteral => Color::LightYellow,
        ElementType::StringLiteral => Color::LightYellow,
        ElementType::_Paren => Color::Gray,
        ElementType::Bracket => Color::Gray,
        ElementType::Brace => Color::Gray,
        ElementType::CollapsedBracket => Color::Gray, // TODO: add fill color
        ElementType::CollapsedBrace => Color::Gray,   // TODO add fill color
    }
}

// taken from https://ratatui.rs/tutorials/json-editor/closing-thoughts/#uirs
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

pub fn inset_chunk(chunk: Rect) -> Rect {
    let chunks = Layout::default()
        .margin(1)
        .constraints([Constraint::Min(1)])
        .split(chunk);
    chunks[0]
}
