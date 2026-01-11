use lazy_static::lazy_static;
use std::{collections::HashMap, rc::Rc};

use crate::token::{Token, TokenType};

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenType> = {
        let mut m = HashMap::new();
        m.insert("and", TokenType::AND);
        m.insert("class", TokenType::CLASS);
        m.insert("else", TokenType::ELSE);
        m.insert("false", TokenType::FALSE);
        m.insert("for", TokenType::FOR);
        m.insert("fun", TokenType::FUN);
        m.insert("if", TokenType::IF);
        m.insert("null", TokenType::NIL);
        m.insert("or", TokenType::OR);
        m.insert("print", TokenType::PRINT);
        m.insert("return", TokenType::RETURN);
        m.insert("super", TokenType::SUPER);
        m.insert("this", TokenType::THIS);
        m.insert("true", TokenType::TRUE);
        m.insert("var", TokenType::VAR);
        m.insert("while", TokenType::WHILE);
        m
    };
}

#[derive(Clone)]
pub struct Scanner {
    source: Rc<Vec<char>>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: Rc<String>) -> Self {
        Self {
            source: Rc::new(source.chars().collect()),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::EOF);
        }

        let c = self.advance();
        match c {
            '(' => self.make_token(TokenType::LeftParen),
            ')' => self.make_token(TokenType::RightParen),
            '{' => self.make_token(TokenType::LeftBrace),
            '}' => self.make_token(TokenType::RightBrace),
            ';' => self.make_token(TokenType::SEMICOLON),
            ',' => self.make_token(TokenType::COMMA),
            '.' => self.make_token(TokenType::DOT),
            '-' => self.make_token(TokenType::MINUS),
            '+' => self.make_token(TokenType::PLUS),
            '*' => self.make_token(TokenType::STAR),
            '!' => {
                let is_equal = self.matches('=');
                self.make_token(if is_equal {
                    TokenType::BangEqual
                } else {
                    TokenType::BANG
                })
            }
            '=' => {
                let is_equal = self.matches('=');
                self.make_token(if is_equal {
                    TokenType::EqualEqual
                } else {
                    TokenType::EQUAL
                })
            }
            '<' => {
                let is_equal = self.matches('=');
                self.make_token(if is_equal {
                    TokenType::LessEqual
                } else {
                    TokenType::LESS
                })
            }
            '>' => {
                let is_equal = self.matches('=');
                self.make_token(if is_equal {
                    TokenType::GreaterEqual
                } else {
                    TokenType::GREATER
                })
            }
            '/' => {
                let is_equal_slash = self.matches('/');
                let is_equal_equal = self.matches('=');

                if is_equal_equal {
                    return self.make_token(TokenType::SlashEqual);
                }

                if !is_equal_slash {
                    self.make_token(TokenType::SLASH)
                } else {
                    panic!("Comment not skipped!");
                }
            }
            '"' => self.string(),
            val if self.is_digit(val) => self.number(),
            val if self.is_alpha(val) => self.identifier(),
            character => self.make_error_token(format!("Unexpected character: '{character}'")),
        }
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        Token::new(token_type, self.line, self.start, self.length(), None, None)
    }

    fn make_literal_token(&self, token_type: TokenType, literal: String) -> Token {
        Token::new(
            token_type,
            self.line,
            self.start,
            self.length(),
            Some(Rc::new(literal)),
            None,
        )
    }

    fn make_error_token(&self, message: String) -> Token {
        Token::new(
            TokenType::Error,
            self.line,
            self.start,
            self.length(),
            None,
            Some(message),
        )
    }

    fn char_at(&self, index: usize) -> char {
        self.source[index]
    }

    fn advance(&mut self) -> char {
        let res = self.char_at(self.current);
        self.current += 1;
        res
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.char_at(self.current) != expected {
            return false;
        }
        self.advance();
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.char_at(self.current)
        }
    }

    fn peek_next(&self) -> char {
        self.source.get(self.current + 1).copied().unwrap_or('\0')
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn substr(&self, start: usize, end: usize) -> String {
        self.source[start..end].iter().collect()
    }

    fn is_digit(&self, c: char) -> bool {
        c.is_ascii_digit()
    }

    fn length(&self) -> usize {
        self.current - self.start
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    fn string(&mut self) -> Token {
        loop {
            let peek = self.peek();

            if peek == '"' || self.is_at_end() {
                break;
            }

            if peek == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            return self.make_error_token("Unclosed string literal".to_owned());
        }
        self.advance();
        let literal = self.substr(self.start + 1, self.current - 1);
        self.make_literal_token(TokenType::STRING, literal)
    }

    fn is_alpha(&self, c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    fn number(&mut self) -> Token {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();
            while self.is_digit(self.peek()) {
                self.advance();
            }
        }
        let literal = self.substr(self.start, self.current);
        self.make_literal_token(TokenType::NUMBER, literal)
    }

    fn identifier(&mut self) -> Token {
        while self.is_alpha(self.peek()) || self.is_digit(self.peek()) {
            self.advance();
        }
        let literal = self.substr(self.start, self.current);
        self.make_literal_token(self.identifier_type(), literal)
    }

    fn identifier_type(&self) -> TokenType {
        let identifier_value = self.substr(self.start, self.current);
        if let Some(token_type) = KEYWORDS.get(identifier_value.as_str()) {
            *token_type
        } else {
            TokenType::IDENTIFIER
        }
    }
}
