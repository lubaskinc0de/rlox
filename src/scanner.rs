use std::collections::HashMap;

use crate::token::{Token, TokenType};

pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: usize,
    keywords: HashMap<String, TokenType>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
            keywords: HashMap::from([
                (String::from("and"), TokenType::AND),
                (String::from("class"), TokenType::CLASS),
                (String::from("else"), TokenType::ELSE),
                (String::from("false"), TokenType::FALSE),
                (String::from("for"), TokenType::FOR),
                (String::from("fn"), TokenType::FUN),
                (String::from("if"), TokenType::IF),
                (String::from("null"), TokenType::NIL),
                (String::from("or"), TokenType::OR),
                (String::from("print"), TokenType::PRINT),
                (String::from("return"), TokenType::RETURN),
                (String::from("super"), TokenType::SUPER),
                (String::from("this"), TokenType::THIS),
                (String::from("true"), TokenType::TRUE),
                (String::from("let"), TokenType::VAR),
                (String::from("while"), TokenType::WHILE),
            ]),
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
            _ => self.make_error_token("Unexpected character".to_owned()),
        }
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        return Token::new(token_type, self.line, self.start, self.length(), None);
    }

    fn make_error_token(&self, message: String) -> Token {
        return Token::new(
            TokenType::Error,
            self.line,
            self.start,
            self.length(),
            Some(message),
        );
    }

    fn char_at(&self, index: usize) -> char {
        self.source
            .chars()
            .nth(index)
            .expect("char index out of range")
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
        return true;
    }

    fn match_prev(&mut self, expected: char) -> bool {
        if self.current == 0 {
            return false;
        }
        if self.char_at(self.current - 1) != expected {
            return false;
        }
        return true;
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.char_at(self.current)
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.chars().count() {
            '\0'
        } else {
            self.char_at(self.current + 1)
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.chars().count()
    }

    fn substr(&self, start: usize, end: usize) -> String {
        let collected: String = self.source.chars().skip(start).take(end - start).collect();
        collected
    }

    fn is_digit(&self, c: char) -> bool {
        return c.is_digit(10);
    }

    fn length(&self) -> usize {
        return self.current - self.start;
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
        self.make_token(TokenType::STRING)
    }

    fn is_alpha(&self, c: char) -> bool {
        return c.is_alphabetic() || c == '_'
    }

    fn number(&mut self) -> Token {
        while self.is_digit(self.peek()) {
            self.advance();
        };

        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();
            while self.is_digit(self.peek()) {
                self.advance();
            };
        }
        self.make_token(TokenType::NUMBER)
    }

    fn identifier(&mut self) -> Token {
        while self.is_alpha(self.peek()) || self.is_digit(self.peek()) {
            self.advance();
        }
        return self.make_token(self.identifier_type())
    }

    fn identifier_type(&self) -> TokenType {
        let identifier_value = self.substr(self.start, self.current);
        let Some(token_type) = self.keywords.get(&identifier_value) else {
            return TokenType::IDENTIFIER
        };
        token_type.clone()
    }
}
