use crate::token::{Token, TokenType};

pub struct Parser {
    pub current: Token,
    pub previous: Token,
}

impl Parser {
    pub fn new() -> Self {
        let eof = Token::new(TokenType::EOF, 1, 0, 0, None, None);
        Self {
            current: eof.clone(),
            previous: eof.clone(),
        }
    }
}
