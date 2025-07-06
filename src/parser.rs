use crate::token::Token;

pub struct Parser {
    pub current: Option<Token>,
    pub previous: Option<Token>,
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
