use derive_more::IsVariant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, IsVariant)]
enum CharType {
    Alpha,
    Symobl,
    Whitespace,
}

impl From<char> for CharType {
    fn from(value: char) -> Self {
        if value.is_whitespace() {
            return Self::Whitespace;
        }
        if value.is_ascii_alphanumeric() {
            return Self::Alpha;
        }
        Self::Symobl
    }
}

pub fn next_word(s: &[char]) -> [i32; 2] {
    pub fn aux(s: &[char], initial: CharType, counter: i32) -> [i32; 2] {
        match s.get(0).copied().map(CharType::from) {
            None => [1, 0],
            Some(ct) => {
                if ct == CharType::Whitespace && initial != CharType::Whitespace {
                    aux(&s[1..], CharType::Whitespace, counter + 1)
                } else if ct == initial {
                    aux(&s[1..], initial, counter + 1)
                } else {
                    [0, counter]
                }
            }
        }
    }

    match s.get(0).copied().map(CharType::from) {
        None => [1, 0],
        Some(ct) => aux(&s[1..], ct, 1),
    }
}

pub fn jump_next_char(s: &[char], c: char, logger: crate::logger::Logger) -> i32 {
    if s.first().copied() == Some(c) {
        s.iter()
            .enumerate()
            .skip(1)
            .find(|(_, cc)| c == **cc)
            .map(|(i, _)| i as _)
            .unwrap_or(0)
    } else {
        s.iter()
            .enumerate()
            .find(|(_, cc)| c == **cc)
            .map(|(i, _)| i as _)
            .unwrap_or(0)
    }
}
