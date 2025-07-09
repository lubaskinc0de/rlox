use std::rc::Rc;

use crate::{
    alias::{StoredChunk, StoredValue, VoidResult},
    chunk::{OpCode, OpCodeKind},
    errors::ParsingError,
    object::string::StringObject,
    parser::Parser,
    rc_refcell,
    scanner::Scanner,
    token::{Literal, Token, TokenType},
    value::Value,
};

use anyhow::Error;
use strum_macros::FromRepr;

#[derive(Debug)]
struct Local {
    name: Rc<Token>,
    depth: usize,
    pub is_initialized: bool,
}

impl Local {
    pub fn new(name: Rc<Token>, depth: usize, is_initialized: bool) -> Self {
        Self {
            name,
            depth,
            is_initialized,
        }
    }

    pub fn mark_initialized(&mut self) {
        self.is_initialized = true;
    }
}

pub struct Compiler {
    parser: Parser,
    scanner: Scanner,
    current_chunk: Option<StoredChunk>,
    debug_mode: bool,
    scope_depth: usize,
    locals: Vec<Local>,
}

#[derive(Copy, Clone, FromRepr, Debug)]
#[allow(clippy::upper_case_acronyms)]
enum Precedence {
    NONE,
    Assignment,
    Or,
    And,
    Eq,
    Cmp,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

type ParseFn = fn(&mut Compiler, can_assign: bool) -> VoidResult;

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
        prefix: Some(Compiler::unary),
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_BANG_EQUAL */
    ParseRule {
        prefix: None,
        infix: Some(Compiler::binary),
        precedence: Eq,
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
        infix: Some(Compiler::binary),
        precedence: Eq,
    },
    /* TOKEN_GREATER */
    ParseRule {
        prefix: None,
        infix: Some(Compiler::binary),
        precedence: Cmp,
    },
    /* TOKEN_GREATER_EQUAL */
    ParseRule {
        prefix: None,
        infix: Some(Compiler::binary),
        precedence: Cmp,
    },
    /* TOKEN_LESS */
    ParseRule {
        prefix: None,
        infix: Some(Compiler::binary),
        precedence: Cmp,
    },
    /* TOKEN_LESS_EQUAL */
    ParseRule {
        prefix: None,
        infix: Some(Compiler::binary),
        precedence: Cmp,
    },
    /* TOKEN_SLASH_EQUAL */
    ParseRule {
        prefix: None,
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_IDENTIFIER */
    ParseRule {
        prefix: Some(Compiler::variable),
        infix: None,
        precedence: NONE,
    },
    /* TOKEN_STRING */
    ParseRule {
        prefix: Some(Compiler::string),
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
        prefix: Some(Compiler::literal),
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
        prefix: Some(Compiler::literal),
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
        prefix: Some(Compiler::literal),
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
    pub fn from_source(source: String, debug_mode: bool) -> Self {
        let scanner = Scanner::new(source);
        let parser = Parser::new();
        Self {
            parser,
            scanner,
            current_chunk: None,
            debug_mode,
            scope_depth: 0,
            locals: vec![],
        }
    }

    pub fn compile(&mut self, chunk: StoredChunk) -> VoidResult {
        self.current_chunk = Some(chunk.clone());

        self.advance()?;
        while !self.matches(&TokenType::EOF)? {
            self.declaration()?;
        }
        Ok(())
    }

    fn previous(&self) -> Option<&Rc<Token>> {
        self.parser.previous.as_ref()
    }

    fn current(&self) -> Option<&Rc<Token>> {
        self.parser.current.as_ref()
    }

    fn debug_string(&self) -> String {
        match (self.current(), self.previous()) {
            (None, None) => String::from("current: None, previous: None"),
            (None, Some(prev)) => format!("current: None, previous: {prev}"),
            (Some(curr), None) => format!("current: {curr}, previous: None"),
            (Some(curr), Some(prev)) => {
                format!("current: {curr}, previous: {prev}")
            }
        }
    }

    fn advance(&mut self) -> VoidResult {
        self.parser.previous = self.current().cloned();
        let new_token = self.scanner.scan_token();

        let message: Option<String> = match new_token.token_type {
            TokenType::Error => Some(new_token.message.clone().unwrap()),
            _ => None,
        };

        self.parser.current = Some(Rc::new(new_token));
        if self.debug_mode {
            println!("Called advance(), {}", self.debug_string(),);
        }

        match self.current().unwrap().token_type {
            TokenType::Error => Err(self.error_at_current(message.unwrap())),
            _ => Ok(()),
        }
    }

    fn error_at_current(&self, message: String) -> Error {
        self.error_at(self.current().unwrap(), message)
    }

    fn error(&self, message: String) -> Error {
        self.error_at(self.previous().unwrap(), message)
    }

    fn error_at(&self, token: &Token, message: String) -> Error {
        print!("[line {}] Error", token.line);
        match token.token_type {
            TokenType::EOF => print!(" at end"),
            TokenType::Error => {}
            _ => print!(
                " at '{}'",
                self.scanner.substr(token.start, token.start + token.length)
            ),
        };
        println!(": {message}");
        ParsingError {}.into()
    }

    fn consume(&mut self, token_type: TokenType, message: String) -> VoidResult {
        if self.current().unwrap().token_type == token_type {
            self.advance()
        } else {
            Err(self.error_at_current(message))
        }
    }

    fn emit_op_code(&self, kind: OpCodeKind) {
        if self.debug_mode {
            println!("Emitted opcode: {kind}")
        }
        self.current_chunk
            .as_ref()
            .unwrap()
            .borrow_mut()
            .push(OpCode::new(kind, self.line()));
    }

    fn emit_const(&self, value: StoredValue) {
        self.emit_op_code(OpCodeKind::Const {
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
        self.previous().unwrap().line
    }

    fn previous_string_literal(&self) -> Result<Literal, Error> {
        if self.previous().unwrap().token_type != TokenType::IDENTIFIER {
            return Err(self.error("Expected identifier".to_owned()));
        }
        let Some(literal) = self.previous().unwrap().literal.clone() else {
            return Err(self.error("Expected literal".to_owned()));
        };
        Ok(literal)
    }

    fn check(&self, token_type: &TokenType) -> bool {
        &self.current().unwrap().token_type == token_type
    }

    fn matches(&mut self, token_type: &TokenType) -> Result<bool, Error> {
        if !self.check(token_type) {
            Ok(false)
        } else {
            self.advance()?;
            Ok(true)
        }
    }

    fn declaration(&mut self) -> VoidResult {
        self.statement()
    }

    fn statement(&mut self) -> VoidResult {
        if self.matches(&TokenType::PRINT)? {
            self.print_statement()
        } else if self.matches(&TokenType::VAR)? {
            self.var_statement()
        } else if self.matches(&TokenType::LeftBrace)? {
            self.begin_scope();
            self.block()?;
            self.end_scope();
            Ok(())
        } else {
            self.expr_statement()
        }
    }

    fn last_local(&mut self) -> Option<&mut Local> {
        let local_count = self.local_count();
        if local_count == 0 {
            return None;
        }
        self.locals.get_mut(local_count - 1)
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn local_count(&self) -> usize {
        self.locals.len()
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        while self.local_count() > 0 && self.last_local().unwrap().depth > self.scope_depth {
            // removing locals of exited scope
            self.emit_op_code(OpCodeKind::Pop);
            self.locals.pop();
        }
    }

    fn is_local_scope(&self) -> bool {
        self.scope_depth > 0
    }

    fn is_global_scope(&self) -> bool {
        self.scope_depth == 0
    }

    fn block(&mut self) -> VoidResult {
        while !self.check(&TokenType::RightBrace) && !self.check(&TokenType::EOF) {
            self.declaration()?;
        }

        self.consume(
            TokenType::RightBrace,
            "Expected '}' at the end of block".to_owned(),
        )
    }

    fn expr_statement(&mut self) -> VoidResult {
        self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expected ';'".to_owned())?;
        self.emit_op_code(OpCodeKind::Pop);
        Ok(())
    }

    fn print_statement(&mut self) -> VoidResult {
        self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expected ';'".to_owned())?;
        self.emit_op_code(OpCodeKind::Print);
        Ok(())
    }

    fn var_statement(&mut self) -> VoidResult {
        let global = self.parse_variable_name("Expected variable name".to_owned())?;

        if self.matches(&TokenType::EQUAL)? {
            self.expression()?
        } else {
            self.emit_op_code(OpCodeKind::Null);
        }

        self.consume(
            TokenType::SEMICOLON,
            "Expected ';' after variable declaration".to_owned(),
        )?;
        self.define_global(global);
        Ok(())
    }

    fn identifier_constant(&mut self, literal: Literal) -> usize {
        self.make_const(rc_refcell!(Value::Identifier(literal,)))
    }

    fn parse_variable_name(&mut self, message: String) -> Result<usize, Error> {
        self.consume(TokenType::IDENTIFIER, message)?;
        self.declare_variable()?;

        if self.is_local_scope() {
            return Ok(0);
            // At runtime, locals aren’t looked up by name.
            // There’s no need to stuff the variable’s name into the constant table,
            // so if the declaration is inside a local scope, we return a dummy table index instead.
        }

        Ok(self.identifier_constant(self.previous_string_literal()?))
    }

    fn declare_variable(&mut self) -> VoidResult {
        if self.is_global_scope() {
            return Ok(());
        }

        let local_name = self.previous().unwrap();
        if local_name.literal.is_none() {
            return Err(self.error("Expected literal".to_owned()));
        }
        self.add_local(local_name.clone());
        Ok(())
    }

    fn add_local(&mut self, name: Rc<Token>) {
        let local = Local::new(name, self.scope_depth, false);
        self.locals.push(local);
    }

    fn define_global(&mut self, name_idx: usize) {
        if self.is_local_scope() {
            self.last_local().unwrap().mark_initialized();
            return;
        }
        self.emit_op_code(OpCodeKind::DefineGlobal { name_idx });
    }

    fn variable(&mut self, can_assign: bool) -> VoidResult {
        self.named_variable(self.previous_string_literal()?, can_assign)
    }

    #[allow(clippy::unnecessary_unwrap)]
    #[allow(clippy::needless_late_init)]
    fn named_variable(&mut self, name: Literal, can_assign: bool) -> VoidResult {
        let get_op: OpCodeKind;
        let set_op: OpCodeKind;

        let local_idx = self.resolve_local(&name)?;
        if local_idx.is_some() {
            get_op = OpCodeKind::ReadLocal {
                name_idx: local_idx.unwrap(),
            };
            set_op = OpCodeKind::SetLocal {
                name_idx: local_idx.unwrap(),
            }
        } else {
            let name_idx = self.identifier_constant(name);
            get_op = OpCodeKind::ReadGlobal { name_idx };
            set_op = OpCodeKind::SetGlobal { name_idx }
        }

        if can_assign && self.matches(&TokenType::EQUAL)? {
            self.expression()?;
            self.emit_op_code(set_op);
        } else {
            self.emit_op_code(get_op);
        }
        Ok(())
    }

    fn resolve_local(&self, name: &Literal) -> Result<Option<usize>, Error> {
        if self.is_global_scope() {
            return Ok(None);
        }
        
        for i in (0..self.local_count()).rev() {
            let local = &self.locals[i];
            if local.name.literal.as_ref().is_some_and(|x| x == name) {
                if !local.is_initialized {
                    return Err(self
                        .error("Cannot read local variable in their own initializer".to_owned()));
                }
                return Ok(Some(i));
            }
        }
        Ok(None)
    }

    fn expression(&mut self) -> VoidResult {
        if self.debug_mode {
            println!("Called expression(), {}", self.debug_string());
        }
        self.parse_precedence(Precedence::Assignment)
    }

    #[allow(unused_variables)]
    fn number(&mut self, can_assign: bool) -> VoidResult {
        let value = Value::Float(
            self.previous()
                .unwrap()
                .literal
                .as_ref()
                .unwrap()
                .parse::<f64>()
                .unwrap(),
        );
        if self.debug_mode {
            println!("Called number() for {value}");
        }
        self.emit_const(rc_refcell!(value));
        Ok(())
    }

    #[allow(unused_variables)]
    fn literal(&mut self, can_assign: bool) -> VoidResult {
        if self.debug_mode {
            println!("Called literal()");
        }
        self.emit_op_code(match self.previous().unwrap().token_type {
            TokenType::NIL => OpCodeKind::Null,
            TokenType::FALSE => OpCodeKind::False,
            TokenType::TRUE => OpCodeKind::True,
            _ => unreachable!(),
        });
        Ok(())
    }

    #[allow(unused_variables)]
    fn string(&mut self, can_assign: bool) -> VoidResult {
        if self.debug_mode {
            println!("Called string()");
        };
        self.emit_const(rc_refcell!(Value::Object(Box::new(StringObject::new(
            self.previous().unwrap().literal.clone().unwrap()
        )))));
        Ok(())
    }

    #[allow(unused_variables)]
    fn grouping(&mut self, can_assign: bool) -> VoidResult {
        self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')'".to_owned())
    }

    #[allow(unused_variables)]
    fn unary(&mut self, can_assign: bool) -> VoidResult {
        let op_type = &self.previous().unwrap().token_type.clone();
        if self.debug_mode {
            println!("Called unary for op {:?}, {}", op_type, self.debug_string(),)
        }

        self.parse_precedence(Precedence::Unary)?;

        match op_type {
            TokenType::MINUS => self.emit_op_code(OpCodeKind::Negate),
            TokenType::BANG => self.emit_op_code(OpCodeKind::Not),
            _ => unreachable!(),
        };
        Ok(())
    }

    fn next_precedence(&self, variant: Precedence) -> Precedence {
        let code = variant as usize;
        Precedence::from_repr(code + 1).unwrap_or(Precedence::Assignment)
    }

    fn get_rule(&self, token_type: &TokenType) -> &ParseRule {
        let idx = *token_type as usize;

        (RULES.get(idx).unwrap()) as _
    }

    #[allow(unused_variables)]
    fn binary(&mut self, can_assign: bool) -> VoidResult {
        let op_type = &self.previous().unwrap().token_type.clone();
        let rule = self.get_rule(op_type);
        let next_precedence = self.next_precedence(rule.precedence);

        if self.debug_mode {
            println!(
                "Called binary {:?}, {}, next precedence = {:?}",
                op_type,
                self.debug_string(),
                next_precedence
            )
        }

        self.parse_precedence(next_precedence)?;

        match op_type {
            TokenType::PLUS => {
                self.emit_op_code(OpCodeKind::Add);
                Ok(())
            }
            TokenType::MINUS => {
                self.emit_op_code(OpCodeKind::Sub);
                Ok(())
            }
            TokenType::SLASH => {
                self.emit_op_code(OpCodeKind::Div);
                Ok(())
            }
            TokenType::STAR => {
                self.emit_op_code(OpCodeKind::Mul);
                Ok(())
            }
            TokenType::BangEqual => {
                self.emit_op_code(OpCodeKind::Eq);
                self.emit_op_code(OpCodeKind::Not);
                Ok(())
            }
            TokenType::EqualEqual => {
                self.emit_op_code(OpCodeKind::Eq);
                Ok(())
            }
            TokenType::GREATER => {
                self.emit_op_code(OpCodeKind::Gt);
                Ok(())
            }
            TokenType::LESS => {
                self.emit_op_code(OpCodeKind::Lt);
                Ok(())
            }
            TokenType::GreaterEqual => {
                self.emit_op_code(OpCodeKind::Lt);
                self.emit_op_code(OpCodeKind::Not);
                Ok(())
            }
            TokenType::LessEqual => {
                self.emit_op_code(OpCodeKind::Gt);
                self.emit_op_code(OpCodeKind::Not);
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> VoidResult {
        if self.debug_mode {
            println!(
                "Called parse_precedence() with precedence = {:?}, {}",
                precedence,
                self.debug_string(),
            )
        }
        self.advance()?;
        let Some(prefix_rule) = self.get_rule(&self.previous().unwrap().token_type).prefix else {
            return Err(self.error("Expected expression".to_owned()));
        };

        let can_assign = precedence as usize <= Assignment as usize;
        prefix_rule(self, can_assign)?;

        let current_token_precedence = self
            .get_rule(&self.current().unwrap().token_type)
            .precedence as usize;

        if (precedence as usize > current_token_precedence) && self.debug_mode {
            println!(
                "Skipping infix rule loop, {}, precedence: {:?}({}), current precedence: {:?}({})",
                self.debug_string(),
                precedence,
                precedence as usize,
                self.get_rule(&self.current().unwrap().token_type)
                    .precedence,
                self.get_rule(&self.current().unwrap().token_type)
                    .precedence as usize,
            );
        }

        let _: () = while (precedence as usize)
            <= (self
                .get_rule(&self.current().unwrap().token_type)
                .precedence as usize)
        {
            if self.debug_mode {
                println!(
                    "Inside infix rule loop, precedence: {:?}({}), current precedence: {:?}({}), {}",
                    precedence,
                    precedence as usize,
                    self.get_rule(&self.current().unwrap().token_type)
                        .precedence,
                    self.get_rule(&self.current().unwrap().token_type)
                        .precedence as usize,
                    self.debug_string()
                )
            }
            self.advance()?;
            let Some(infix_rule) = self.get_rule(&self.previous().unwrap().token_type).infix else {
                continue;
            };
            if self.debug_mode {
                println!("Calling infix rule for {}", self.previous().unwrap())
            }
            infix_rule(self, can_assign)?;
        };

        if can_assign && self.matches(&TokenType::EQUAL)? {
            Err(self.error("Invalid assignment target".to_owned()))
        } else {
            Ok(())
        }
    }
}
