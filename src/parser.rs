use crate::token::Token;

pub struct Parser {
    pub current: Option<Token>,
    pub previous: Option<Token>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            current: None,
            previous: None,
        }
    }
}
