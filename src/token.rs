#[derive(Debug, Clone)]
pub enum TokenType {
    // Single-character tokens.
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

    // One or two character tokens.
    BANG,
    BangEqual,
    EQUAL,
    EqualEqual,
    GREATER,
    GreaterEqual,
    LESS,
    LessEqual,
    Pow,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    PowEqual,

    // Literals.
    IDENTIFIER,
    STRING,
    NUMBER,

    // Keywords.
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

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub start: usize,
    pub length: usize,
    pub message: Option<String>,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        line: usize,
        start: usize,
        length: usize,
        message: Option<String>,
    ) -> Self {
        match token_type {
            TokenType::Error => {
                if message.is_none() {
                    panic!("Error token with empty message")
                }
            },
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
        }
    }
}
