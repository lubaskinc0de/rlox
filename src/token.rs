#[derive(Debug, Clone, PartialEq, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,
    BANG,
    BangEqual,
    EQUAL,
    EqualEqual,
    GREATER,
    GreaterEqual,
    LESS,
    LessEqual,
    SlashEqual,
    IDENTIFIER,
    STRING,
    NUMBER,
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,
    EOF,
    Error,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub start: usize,
    pub length: usize,
    pub literal: Option<String>,
    pub message: Option<String>,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        line: usize,
        start: usize,
        length: usize,
        literal: Option<String>,
        message: Option<String>,
    ) -> Self {
        match token_type {
            TokenType::Error => {
                if message.is_none() {
                    panic!("Error token with empty message")
                }
            }
            _ => {
                if message.is_some() {
                    panic!("Non-error token with message")
                }
            }
        }
        Self {
            token_type,
            line,
            start,
            length,
            message,
            literal,
        }
    }
}
