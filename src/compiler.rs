use crate::{
    alias::{StoredChunk, StoredValue},
    chunk::OpCode,
    parser::Parser,
    rc_refcell,
    scanner::Scanner,
    token::{Token, TokenType},
    value::Value,
};

use strum_macros::FromRepr;

pub struct Compiler {
    parser: Parser,
    scanner: Scanner,
    current_chunk: Option<StoredChunk>,
}

#[derive(Copy, Clone, FromRepr, Debug)]
enum Precedence {
    NONE,
    Assignment,
    Or,
    And,
    Eq,
    Comp,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

type ParseFn = fn(&mut Compiler);

#[derive(Debug)]
struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: Precedence,
}

use Precedence::*;

const RULES: [ParseRule; 41] = [
    /* TOKEN_LEFT_PAREN */
    ParseRule {
        prefix: Some(Compiler::grouping),
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_RIGHT_PAREN */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_LEFT_BRACE */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_RIGHT_BRACE */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_COMMA */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_DOT */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_MINUS */
    ParseRule {
        prefix: Some(Compiler::unary),
        infix: Some(Compiler::binary),
        precedence: Term,
    },
    /* TOKEN_PLUS */
    ParseRule {
        prefix: None,
        infix: Some(Compiler::binary),
        precedence: Term,
    },
    /* TOKEN_SEMICOLON */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_SLASH */
    ParseRule {
        prefix: None,
        infix: Some(Compiler::binary),
        precedence: Factor,
    },
    /* TOKEN_STAR */
    ParseRule {
        prefix: None,
        infix: Some(Compiler::binary),
        precedence: Factor,
    },
    /* TOKEN_BANG */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_BANG_EQUAL */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_EQUAL */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_EQUAL_EQUAL */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_GREATER */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_GREATER_EQUAL */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_LESS */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_LESS_EQUAL */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_SLASH_EQUAL */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_IDENTIFIER */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_STRING */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_NUMBER */
    ParseRule {
        prefix: Some(Compiler::number),
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_AND */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_CLASS */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_ELSE */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_FALSE */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_FOR */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_FUN */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_IF */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_NIL */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_OR */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_PRINT */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_RETURN */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_SUPER */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_THIS */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_TRUE */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_VAR */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_WHILE */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_ERROR */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_EOF */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
];

impl Compiler {
    pub fn new(parser: Parser, scanner: Scanner) -> Self {
        Self {
            parser,
            scanner,
            current_chunk: None,
        }
    }

    pub fn from_source(source: String) -> Self {
        let scanner = Scanner::new(source);
        let parser = Parser::new();
        Self {
            parser,
            scanner,
            current_chunk: None,
        }
    }

    pub fn compile(&mut self, chunk: StoredChunk) -> bool {
        self.current_chunk = Some(chunk.clone());
        self.parser.had_error.replace(false);
        self.parser.panic_mode.replace(false);

        self.advance();
        self.expression();
        self.consume(TokenType::EOF, "Expected end of expression".to_owned());
        !self.parser.had_error.get() // is compiled succesfully
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current.clone();

        loop {
            let new_token = self.scanner.scan_token();

            let message: Option<String> = match new_token.token_type {
                TokenType::Error => Some(new_token.message.clone().unwrap()),
                _ => None,
            };

            self.parser.current = new_token;
            match self.parser.current.token_type {
                TokenType::Error => self.error_at_current(message.unwrap()),
                _ => break,
            }
        }
    }

    fn error_at_current(&self, message: String) {
        self.error_at(&self.parser.current, message);
    }

    fn error(&self, message: String) {
        self.error_at(&self.parser.previous, message);
    }

    fn error_at(&self, token: &Token, message: String) {
        if self.parser.panic_mode.get() {
            return;
        }

        self.parser.panic_mode.replace(true);
        print!("[line {}] Error", token.line);
        match token.token_type {
            TokenType::EOF => print!(" at end"),
            TokenType::Error => {}
            _ => print!(
                " at '{}'",
                self.scanner.substr(token.start, token.start + token.length)
            ),
        };
        print!(": {}\n", message);
        self.parser.had_error.replace(true);
    }

    fn consume(&mut self, token_type: TokenType, message: String) {
        if self.parser.current.token_type == token_type {
            self.advance();
        } else {
            self.error_at_current(message);
        }
    }

    fn emit_op_code(&self, op_code: OpCode) {
        self.current_chunk
            .as_ref()
            .unwrap()
            .borrow_mut()
            .push(op_code);
    }

    fn emit_op_codes(&self, op_code_a: OpCode, op_code_b: OpCode) {
        self.current_chunk
            .as_ref()
            .unwrap()
            .borrow_mut()
            .push(op_code_a);

        self.current_chunk
            .as_ref()
            .unwrap()
            .borrow_mut()
            .push(op_code_b);
    }

    fn emit_const(&self, value: StoredValue) {
        self.emit_op_code(OpCode::OpConst {
            line: self.line(),
            const_idx: self.make_const(value),
        });
    }

    fn make_const(&self, value: StoredValue) -> usize {
        self.current_chunk
            .as_ref()
            .unwrap()
            .borrow_mut()
            .push_const(value)
    }

    fn line(&self) -> usize {
        self.parser.previous.line
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        let value = Value::Float(
            self.parser
                .previous
                .literal
                .as_ref()
                .unwrap()
                .parse::<f64>()
                .unwrap(),
        );
        self.emit_const(rc_refcell!(value));
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expected ')'".to_owned());
    }

    fn unary(&mut self) {
        let op_type = &self.parser.previous.token_type.clone();
        self.parse_precedence(Precedence::Unary);

        match op_type {
            TokenType::MINUS => self.emit_op_code(OpCode::OpNegate { line: self.line() }),
            _ => return,
        }
    }

    fn next_precedence(&self, variant: Precedence) -> Precedence {
        let code = variant as usize;
        Precedence::from_repr(code).unwrap_or(Precedence::Assignment)
    }

    fn get_rule(&self, token_type: &TokenType) -> &ParseRule {
        let idx = *token_type as usize;
        let rule = RULES.get(idx).unwrap();
        rule
    }

    fn binary(&mut self) {
        let op_type = &self.parser.previous.token_type.clone();
        let rule = self.get_rule(op_type);
        let next_precedence = self.next_precedence(rule.precedence);
        self.parse_precedence(next_precedence);

        match op_type {
            TokenType::PLUS => self.emit_op_code(OpCode::OpAdd { line: self.line() }),
            TokenType::MINUS => self.emit_op_code(OpCode::OpSub { line: self.line() }),
            TokenType::SLASH => self.emit_op_code(OpCode::OpDiv { line: self.line() }),
            TokenType::STAR => self.emit_op_code(OpCode::OpMul { line: self.line() }),
            _ => panic!("Unsupported binary token"),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let Some(prefix_rule) = self.get_rule(&self.parser.previous.token_type).prefix else {
            self.error("Expected expression".to_owned());
            return;
        };

        prefix_rule(self);

        while (precedence as usize)
            <= (self.get_rule(&self.parser.current.token_type).precedence as usize)
        {
            self.advance();
            let Some(infix_rule) = self.get_rule(&self.parser.previous.token_type).infix else {
                continue;
            };
            infix_rule(self);
        }
    }
}
