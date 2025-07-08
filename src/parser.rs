use std::rc::Rc;

use crate::token::Token;

pub struct Parser {
    pub current: Option<Rc<Token>>,
    pub previous: Option<Rc<Token>>,
}

/// Actual parsing logic in ``Compiler`` according to the book
impl Parser {
    pub fn new() -> Self {
        Self {
            current: None,
            previous: None,
        }
    }
}
