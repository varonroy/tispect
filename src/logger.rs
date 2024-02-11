use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct Logger(Rc<RefCell<Vec<String>>>);

impl Logger {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(Vec::new())))
    }

    pub fn log(&self, s: impl ToString) {
        self.0.borrow_mut().push(s.to_string());
    }

    pub fn get_all_logs(&self) -> Vec<String> {
        self.0.borrow().clone()
    }
}
