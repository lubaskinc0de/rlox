use std::cell::Cell;

use crate::token::{Token, TokenType};

pub struct Parser {
    pub current: Token,
    pub previous: Token,
    pub had_error: Cell<bool>,
    pub panic_mode: Cell<bool>,
}

impl Parser {
    pub fn new() -> Self {
        let eof = Token::new(TokenType::EOF, 1, 0, 0, None);
        Self {
            current: eof.clone(),
            previous: eof.clone(),
            had_error: Cell::new(false),
            panic_mode: Cell::new(false),
        }
    }
}
