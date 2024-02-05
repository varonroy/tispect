use derive_more::IsVariant;

mod contained_value;
mod parser;
pub mod writer;

pub use contained_value::ContainedValue;
pub use parser::parse;

use self::writer::{Element, Writer};

// #[derive(Debug, Clone, Copy, IsVariant)]
// pub enum Number {
//     U64(u64),
//     I64(i64),
//     F64(f64),
// }
//
// impl ToString {
// }

#[derive(Debug, Clone)]
pub struct ValueArray<'a> {
    pub collapse: bool,
    pub arr: Vec<Value<'a>>,
}

impl<'a> ValueArray<'a> {
    pub fn toggle_collapse(&mut self) {
        self.collapse = !self.collapse;
    }

    pub fn collapse(&mut self) {
        self.collapse = true;
    }

    pub fn expand(&mut self) {
        self.collapse = false;
    }
    pub fn get(&self, idx: usize) -> Option<&Value<'a>> {
        self.arr.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Value<'a>> {
        self.arr.get_mut(idx)
    }
}

#[derive(Debug, Clone)]
pub struct ValueObject<'a> {
    pub collapse: bool,
    pub map: Vec<(&'a str, Value<'a>)>,
}

impl<'a> ValueObject<'a> {
    pub fn toggle_collapse(&mut self) {
        self.collapse = !self.collapse;
    }

    pub fn collapse(&mut self) {
        self.collapse = true;
    }

    pub fn expand(&mut self) {
        self.collapse = false;
    }

    pub fn get(&self, key: &str) -> Option<&Value<'a>> {
        self.map
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, item)| item)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value<'a>> {
        self.map
            .iter_mut()
            .find(|(k, _)| *k == key)
            .map(|(_, item)| item)
    }
}

// TODO: add cache
#[derive(Debug, Clone, IsVariant)]
pub enum Value<'a> {
    Null,
    Bool(bool),
    Number(&'a str),
    String(&'a str),
    Array(ValueArray<'a>),
    Object(ValueObject<'a>),
}

impl<'a> Value<'a> {
    pub fn get_null(&self) -> Option<()> {
        match self {
            Self::Null => Some(()),
            _ => None,
        }
    }

    pub fn get_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(x) => Some(*x),
            _ => None,
        }
    }

    pub fn get_number(&self) -> Option<&str> {
        match self {
            Self::Number(x) => Some(*x),
            _ => None,
        }
    }

    pub fn get_string(&self) -> Option<&str> {
        match self {
            Self::String(x) => Some(*x),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&ValueArray> {
        match self {
            Self::Array(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut ValueArray<'a>> {
        match self {
            Self::Array(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&ValueObject> {
        match self {
            Self::Object(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut ValueObject<'a>> {
        match self {
            Self::Object(x) => Some(x),
            _ => None,
        }
    }

    pub fn lines(&self) -> Vec<String> {
        let mut writer = Writer::new();
        writer.write_value(self);
        writer.lines()
    }

    pub fn elemnets(&self) -> Vec<Vec<Element>> {
        let mut writer = Writer::new();
        writer.write_value(self);
        writer.get()
    }
}

impl ToString for Value<'_> {
    fn to_string(&self) -> String {
        let mut writer = Writer::new();
        writer.write_value(self);
        writer.to_string()
    }
}
