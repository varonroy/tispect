use derive_more::IsVariant;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, IsVariant)]
pub enum MatchStatus {
    FullMatch,
    PartialMatch,
    NoMatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IsVariant)]
pub enum PatternItem {
    Token(&'static str),
    Literal(char),
    Number,
    AnyChar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IsVariant)]
enum PatternInputItem {
    Token(&'static str),
    Char(char),
}

impl PatternInputItem {
    fn str_to_item_char_vec(s: &str) -> Vec<Self> {
        s.chars().into_iter().map(Self::Char).collect()
    }

    fn is_char_numeric(self) -> bool {
        match self {
            Self::Char(c) => c.is_numeric(),
            _ => false,
        }
    }
}

impl From<char> for PatternInputItem {
    fn from(value: char) -> Self {
        Self::Char(value)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PatternParseError {
    #[error("Invalid escape character `{0}`")]
    InvalidEscape(char),
}

#[derive(Debug, Clone)]
pub struct Pattern(Vec<PatternItem>);

impl FromStr for Pattern {
    type Err = PatternParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut v = Vec::new();
        let mut escape = false;
        for c in s.chars() {
            if c == '\\' {
                escape = true;
                continue;
            }
            if escape {
                v.push(match c {
                    '\\' => PatternItem::Literal('\\'),
                    'n' => PatternItem::Literal('\n'),
                    't' => PatternItem::Literal('\t'),
                    'd' => PatternItem::Number,
                    'c' => PatternItem::AnyChar,
                    _ => {
                        return Err(PatternParseError::InvalidEscape(c));
                    }
                });
                escape = false;
                continue;
            }

            v.push(PatternItem::Literal(c));
        }

        Ok(Self(v))
    }
}

impl Pattern {
    fn match_aux(pattern: &[PatternItem], source: &[PatternInputItem]) -> MatchStatus {
        if pattern.is_empty() && source.is_empty() {
            return MatchStatus::FullMatch;
        }

        // example:
        //   pattern: g
        //   string:  gg
        if pattern.is_empty() && !source.is_empty() {
            return MatchStatus::NoMatch;
        }

        // example:
        //   pattern: gg
        //   string:  g
        if !pattern.is_empty() && source.is_empty() {
            return MatchStatus::PartialMatch;
        }

        let Some(p) = pattern.first().copied() else {
            return MatchStatus::NoMatch;
        };

        let Some(s) = source.first().copied() else {
            return MatchStatus::NoMatch;
        };

        match (p, s) {
            (PatternItem::Token(t), PatternInputItem::Token(s)) if t == s => {
                Self::match_aux(&pattern[1..], &source[1..])
            }
            (PatternItem::Literal(c), PatternInputItem::Char(s)) if c == s => {
                return Self::match_aux(&pattern[1..], &source[1..])
            }
            (PatternItem::Number, PatternInputItem::Char(s)) if s.is_numeric() => {
                let mut source = source;
                while source.first().map(|c| c.is_char_numeric()).unwrap_or(false) {
                    source = &source[1..];
                }
                Self::match_aux(&pattern[1..], source)
            }
            (PatternItem::AnyChar, PatternInputItem::Char(_)) => {
                Self::match_aux(&pattern[1..], &source[1..])
            }
            _ => MatchStatus::NoMatch,
        }
    }

    pub fn match_str(&self, s: &str) -> MatchStatus {
        let s = PatternInputItem::str_to_item_char_vec(s);
        Self::match_aux(&self.0, &s)
    }
}

#[cfg(test)]
mod tests {
    use crate::simple_matcher::PatternItem;

    use super::{MatchStatus, Pattern};

    fn assert_match<E: std::error::Error>(p: Result<Pattern, E>, s: &str, status: MatchStatus) {
        let p = p.expect("Could not parse error: {p:?}");

        let got = p.match_str(s);
        assert!(
            got == status,
            "Statuses don't match. p: `{p:?}`, s: `{s:?}`, got: `{got:?}`, expected: `{status:?}`"
        );
    }

    #[test]
    fn parse() {
        type I = PatternItem;

        assert_eq!(
            "j".parse::<Pattern>().unwrap().0,
            Pattern(vec![I::Literal('j')]).0
        );

        assert_eq!(
            "jj".parse::<Pattern>().unwrap().0,
            Pattern(vec![I::Literal('j'), I::Literal('j')]).0
        );

        assert_eq!(
            "f\\c".parse::<Pattern>().unwrap().0,
            Pattern(vec![I::Literal('f'), I::AnyChar]).0
        );
    }

    #[test]
    fn test() {
        assert_match("j".parse::<Pattern>(), "j", MatchStatus::FullMatch);

        assert_match("jj".parse::<Pattern>(), "jj", MatchStatus::FullMatch);

        assert_match("f\\c".parse::<Pattern>(), "A", MatchStatus::NoMatch);
        assert_match("f\\c".parse::<Pattern>(), "fX", MatchStatus::FullMatch);
        assert_match("f\\c".parse::<Pattern>(), "f\"", MatchStatus::FullMatch);

        assert_match("gg".parse::<Pattern>(), "fX", MatchStatus::NoMatch);
        assert_match("gg".parse::<Pattern>(), "g", MatchStatus::PartialMatch);
        assert_match("gg".parse::<Pattern>(), "gg", MatchStatus::FullMatch);
        assert_match("gg".parse::<Pattern>(), "ggg", MatchStatus::NoMatch);
    }
}
