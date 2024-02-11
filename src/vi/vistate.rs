use crate::simple_matcher::{self, MatchStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViCommand {
    // simple navigation
    Up,
    Down,
    Left,
    Right,
    MoveWordForward,
    MoveWordBackward,
    // jumps
    JumpNextChar(char),
    JumpPreviousChar(char),
    // global movement
    FirstLine,
    LastLine,
    FirstColumn,
    LastColumn,
    // search results
    Next,
    Previous,
}

trait ViCommandBuilder {
    fn build(&self, s: &str) -> ViCommand;
}

impl ViCommandBuilder for ViCommand {
    fn build(&self, _: &str) -> ViCommand {
        *self
    }
}

impl<F> ViCommandBuilder for F
where
    F: Fn(&str) -> ViCommand,
{
    fn build(&self, s: &str) -> ViCommand {
        self(s)
    }
}

pub struct ViState {
    pending: String,
    patterns: Vec<(simple_matcher::Pattern, Box<dyn ViCommandBuilder>)>,
}

impl ViState {
    fn create_patterns() -> Result<
        Vec<(simple_matcher::Pattern, Box<dyn ViCommandBuilder>)>,
        simple_matcher::PatternParseError,
    > {
        type C = ViCommand;
        Ok(vec![
            // simple navigation
            ("h".parse()?, Box::new(C::Left)),
            ("j".parse()?, Box::new(C::Down)),
            ("k".parse()?, Box::new(C::Up)),
            ("l".parse()?, Box::new(C::Right)),
            ("w".parse()?, Box::new(C::MoveWordForward)),
            ("b".parse()?, Box::new(C::MoveWordBackward)),
            // jumps
            (
                "f\\c".parse()?,
                Box::new(|s: &str| C::JumpNextChar(s.chars().last().unwrap())),
            ),
            (
                "F\\c".parse()?,
                Box::new(|s: &str| C::JumpPreviousChar(s.chars().last().unwrap())),
            ),
            // global movement
            ("gg".parse()?, Box::new(C::FirstLine)),
            ("G".parse()?, Box::new(C::LastLine)),
            ("0".parse()?, Box::new(C::FirstColumn)),
            ("$".parse()?, Box::new(C::LastColumn)),
            // search results
            ("n".parse()?, Box::new(C::Next)),
            ("p".parse()?, Box::new(C::Previous)),
        ])
    }

    pub fn new() -> Self {
        Self {
            pending: String::new(),
            patterns: Self::create_patterns().unwrap(),
        }
    }

    pub fn reset(&mut self) {
        self.pending.clear();
    }

    pub fn process(&mut self, c: char) -> Option<ViCommand> {
        self.pending.push(c);
        let mut has_partial_matche = false;
        for (pattern, builder) in &self.patterns {
            match pattern.match_str(&self.pending) {
                MatchStatus::NoMatch => {}
                MatchStatus::PartialMatch => {
                    has_partial_matche = true;
                }
                MatchStatus::FullMatch => {
                    let command = builder.build(&self.pending);
                    self.pending = String::new();
                    return Some(command);
                }
            }
        }
        if has_partial_matche == false {
            self.pending.clear();
        }
        None
    }
}
