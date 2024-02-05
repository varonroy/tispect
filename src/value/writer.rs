use super::{Value, ValueArray, ValueObject};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    Whtiespace,
    Comma,
    Key,
    NullLiteral,
    BoolLiteral,
    NumberLiteral,
    StringLiteral,
    _Paren,           // `(` or `)`
    Bracket,          // `[` or `]`
    Brace,            // `{` or `}`
    CollapsedBracket, // `[ ... ]`
    CollapsedBrace,   // `{ ... }`
}

#[derive(Debug, Clone)]
pub struct Element {
    pub ty: ElementType,
    pub content: String,
}

impl Element {
    fn new(ty: ElementType, content: impl ToString) -> Self {
        Self {
            ty,
            content: content.to_string(),
        }
    }

    fn whtiespace(s: impl ToString) -> Self {
        Self::new(ElementType::Whtiespace, s)
    }

    fn comma() -> Self {
        Self::new(ElementType::Comma, ",")
    }

    fn key(s: impl ToString) -> Self {
        Self::new(ElementType::Key, s)
    }

    fn null_literal() -> Self {
        Self::new(ElementType::NullLiteral, "null")
    }

    fn bool_literal(value: bool) -> Self {
        Self::new(ElementType::BoolLiteral, value.to_string())
    }

    fn number_literal(s: impl ToString) -> Self {
        Self::new(ElementType::NumberLiteral, s)
    }

    fn string_literal(s: impl ToString) -> Self {
        Self::new(ElementType::StringLiteral, s)
    }

    fn _paren(s: impl ToString) -> Self {
        Self::new(ElementType::_Paren, s)
    }

    fn open_bracket() -> Self {
        Self::new(ElementType::Bracket, "[")
    }

    fn close_bracket() -> Self {
        Self::new(ElementType::Bracket, "]")
    }

    fn open_brace() -> Self {
        Self::new(ElementType::Brace, "{")
    }

    fn close_brace() -> Self {
        Self::new(ElementType::Brace, "}")
    }

    fn collapsed_bracket() -> Self {
        Self::new(ElementType::CollapsedBracket, "[ ... ]")
    }

    fn collapsed_brace() -> Self {
        Self::new(ElementType::CollapsedBrace, "{ ... }")
    }
}

pub struct Writer {
    buffer: Vec<Vec<Element>>,
    indent_str: String,
    indent: u32,
}

impl Writer {
    pub fn new() -> Self {
        Self {
            buffer: vec![vec![]],
            indent_str: "    ".to_string(),
            indent: 0,
        }
    }

    pub fn lines(&self) -> Vec<String> {
        self.buffer
            .iter()
            .map(|v| {
                v.into_iter()
                    .map(|e| e.content.clone())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect()
    }

    pub fn with_indent_str(&mut self, indent_str: impl ToString) {
        self.indent_str = indent_str.to_string();
    }

    pub fn with_indent_size(&mut self, indent_size: usize) {
        self.indent_str = vec![" "; indent_size].join(" ");
    }

    fn push_last(&mut self, e: Element) {
        self.buffer.last_mut().unwrap().push(e);
    }

    fn newline(&mut self) {
        self.buffer.push(Vec::new());
        for _ in 0..(self.indent as usize) {
            self.push_last(Element::whtiespace(self.indent_str.clone()));
        }
    }

    fn write_null(&mut self) {
        self.push_last(Element::null_literal());
    }

    fn write_bool(&mut self, value: bool) {
        self.push_last(Element::bool_literal(value));
    }

    // fn write_number(&mut self, value: &Number) {
    //     self.push_last(value.to_string())
    // }

    fn write_number(&mut self, value: &str) {
        self.push_last(Element::number_literal(value));
    }

    fn write_string(&mut self, value: impl ToString) {
        let value = value.to_string();
        self.push_last(Element::string_literal(format!("\"{value}\"")));
    }

    fn write_arr(&mut self, arr: &[Value], collapse: bool) {
        if collapse {
            self.push_last(Element::collapsed_bracket());
            return;
        }

        let short = arr.len() < 5;
        let simple = arr
            .iter()
            .find(|value| value.is_array() || value.is_object())
            .is_none();
        let expanded = !short || !simple;

        self.push_last(Element::open_bracket());

        if expanded {
            self.indent += 1;
            self.newline();
        }

        for i in 0..arr.len() {
            self.write_value(&arr[i]);

            if i != arr.len() - 1 {
                self.push_last(Element::comma());
            }

            if i == arr.len() - 1 {
                if expanded {
                    self.indent -= 1;
                }
            }

            if expanded {
                self.newline();
            }
        }

        // if expanded {
        //     self.indent -= 1;
        // }

        self.push_last(Element::close_bracket());
    }

    fn write_object(&mut self, object: &[(&str, Value)], collapse: bool) {
        if collapse {
            self.push_last(Element::collapsed_brace());
            return;
        }

        let short = object.len() < 2;
        let simple = object
            .iter()
            .find(|(_, value)| value.is_array() || value.is_object())
            .is_none();
        let expanded = !short || !simple;

        self.push_last(Element::open_brace());

        if expanded {
            self.indent += 1;
            self.newline();
        }

        for i in 0..object.len() {
            self.push_last(Element::key(format!("\"{}\": ", object[i].0)));

            self.write_value(&object[i].1);

            if i != object.len() - 1 {
                self.push_last(Element::comma());
            }

            if i == object.len() - 1 {
                if expanded {
                    self.indent -= 1;
                }
            }

            if expanded {
                self.newline();
            }
        }

        // if expanded {
        //     self.indent -= 1;
        // }

        self.push_last(Element::close_brace());
    }

    pub fn write_value(&mut self, value: &Value) {
        match value {
            Value::Null => self.write_null(),
            Value::Bool(x) => self.write_bool(*x),
            Value::Number(x) => self.write_number(x),
            Value::String(x) => self.write_string(x),
            Value::Array(ValueArray { collapse, arr }) => self.write_arr(arr.as_slice(), *collapse),
            Value::Object(ValueObject { collapse, map }) => {
                self.write_object(map.as_slice(), *collapse)
            }
        }
    }

    pub fn get(self) -> Vec<Vec<Element>> {
        self.buffer
    }
}

impl ToString for Writer {
    fn to_string(&self) -> String {
        self.buffer
            .iter()
            .map(|line| {
                line.iter()
                    .map(|e| e.content.as_str())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
