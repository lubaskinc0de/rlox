use crate::{
    alias::{StoredChunk, StoredObject, StoredValue, VoidResult},
    chunk::{OpCode, OpCodeKind},
    errors::ParsingError,
    object::{function::FunctionObject, string::StringObject},
    rc_refcell,
    scanner::Scanner,
    token::{Literal, Token, TokenType},
    value::Value,
};
use Precedence::*;
use log::debug;
use std::rc::Rc;

use anyhow::{Error, Ok};
use strum_macros::FromRepr;

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

type ParseFn<'scanner> = fn(&mut Compiler<'scanner>, can_assign: bool) -> VoidResult;
type Rules<'scanner> = [ParseRule<'scanner>; 41];

#[derive(Debug, Clone)]
struct ParseRule<'scanner> {
    prefix: Option<ParseFn<'scanner>>,
    infix: Option<ParseFn<'scanner>>,
    precedence: Precedence,
}

#[derive(Debug)]
pub struct Local {
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

#[derive(Clone)]
pub enum FunctionType {
    Script,
    Function,
}

pub struct CompilerState {
    function: StoredObject<FunctionObject>,
    function_type: FunctionType,
    locals: Vec<Local>,
    scope_depth: usize,
}

impl CompilerState {
    pub fn new(function_type: FunctionType, function_name: Literal) -> Self {
        Self {
            function: rc_refcell!(FunctionObject::new(0, function_name)),
            function_type,
            locals: vec![],
            scope_depth: 0,
        }
    }

    pub fn with_chunk(
        function_type: FunctionType,
        function_name: Literal,
        chunk: StoredChunk,
    ) -> Self {
        Self {
            function: rc_refcell!(FunctionObject {
                arity: 0,
                chunk,
                name: function_name
            }),
            function_type,
            locals: vec![],
            scope_depth: 0,
        }
    }
}

pub struct Compiler<'scanner> {
    scanner: &'scanner mut Scanner,
    current: Option<Rc<Token>>,
    previous: Option<Rc<Token>>,
    states: Vec<CompilerState>,
    rules: Rules<'scanner>,
}

impl<'scanner> Compiler<'scanner> {
    pub fn new(scanner: &'scanner mut Scanner, chunk: StoredChunk) -> Self {
        Self {
            scanner,
            current: None,
            previous: None,
            states: vec![CompilerState::with_chunk(
                FunctionType::Script,
                Rc::new("".to_owned()),
                chunk,
            )],
            rules: Compiler::build_rules(),
        }
    }

    pub fn compile(&mut self) -> Result<StoredObject<FunctionObject>, Error> {
        self.advance()?;
        while !self.matches(&TokenType::EOF)? {
            self.declaration()?;
        }
        Ok(self.func())
    }

    fn state(&self) -> &CompilerState {
        self.states.last().unwrap()
    }

    fn state_mut(&mut self) -> &mut CompilerState {
        self.states.last_mut().unwrap()
    }

    fn func(&self) -> StoredObject<FunctionObject> {
        self.state().function.clone()
    }

    fn current_chunk(&self) -> StoredChunk {
        self.func().borrow().chunk.clone()
    }

    fn previous(&self) -> Option<&Rc<Token>> {
        self.previous.as_ref()
    }

    fn current(&self) -> Option<&Rc<Token>> {
        self.current.as_ref()
    }

    fn locals(&self) -> &Vec<Local> {
        &self.state().locals
    }

    fn locals_mut(&mut self) -> &mut Vec<Local> {
        &mut self.state_mut().locals
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
        self.previous = self.current().cloned();
        let new_token = self.scanner.scan_token();

        let message: Option<String> = match new_token.token_type {
            TokenType::Error => Some(new_token.message.clone().unwrap()),
            _ => None,
        };

        self.current = Some(Rc::new(new_token));
        debug!("Called advance(), {}", self.debug_string(),);

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
        debug!("Emitted opcode: {kind}");
        self.current_chunk()
            .borrow_mut()
            .push(OpCode::new(kind, self.line()));
    }

    fn emit_const(&self, value: StoredValue) {
        self.emit_op_code(OpCodeKind::Const {
            const_idx: self.make_const(value),
        });
    }

    fn make_const(&self, value: StoredValue) -> usize {
        self.current_chunk().borrow_mut().push_const(value)
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
        } else if self.matches(&TokenType::IF)? {
            self.if_statement()
        } else if self.matches(&TokenType::WHILE)? {
            self.while_statement()
        } else if self.matches(&TokenType::FOR)? {
            self.for_statement()
        } else if self.matches(&TokenType::FUN)? {
            self.func_decl()
        } else if self.matches(&TokenType::RETURN)? {
            self.return_stmt()
        } else {
            self.expr_statement()
        }
    }

    fn last_local(&mut self) -> Option<&mut Local> {
        let local_count = self.local_count();
        if local_count == 0 {
            return None;
        }
        self.locals_mut().get_mut(local_count - 1)
    }

    fn begin_scope(&mut self) {
        self.state_mut().scope_depth += 1;
    }

    fn local_count(&self) -> usize {
        self.locals().len()
    }

    fn end_scope(&mut self) {
        self.state_mut().scope_depth -= 1;
        while self.local_count() > 0 && self.last_local().unwrap().depth > self.state().scope_depth
        {
            // removing locals of exited scope
            self.emit_op_code(OpCodeKind::Pop);
            self.locals_mut().pop();
        }
    }

    fn is_local_scope(&self) -> bool {
        self.state().scope_depth > 0
    }

    fn is_global_scope(&self) -> bool {
        self.state().scope_depth == 0
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
        debug!("Called var statement");
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
        self.define_variable(global);
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
        debug!("Pushing {name} local");
        let local = Local::new(name, self.state().scope_depth, false);
        self.locals_mut().push(local);
    }

    fn define_variable(&mut self, name_idx: usize) {
        debug!("Called define_variable");
        if self.is_local_scope() {
            debug!("Initializing local");
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
            let local = &self.locals()[i];
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

    fn emit_jump(&mut self, kind: OpCodeKind) -> usize {
        self.emit_op_code(kind);
        self.current_chunk().borrow().len() - 1
    }

    fn patch_jump(&mut self, jump_idx: usize) {
        let current_chunk = self.current_chunk();
        let jump = current_chunk.borrow().len() - 1 - jump_idx;
        let mut mut_chunk = current_chunk.borrow_mut();
        let op_code = mut_chunk
            .get(jump_idx)
            .expect("Invalid jump offset in patch_jump()");

        match &mut op_code.kind() {
            OpCodeKind::JumpIfFalse { .. } => {
                mut_chunk.replace(
                    jump_idx,
                    OpCode::new(OpCodeKind::JumpIfFalse { offset: jump }, self.line()),
                );
            }
            OpCodeKind::Jump { .. } => {
                mut_chunk.replace(
                    jump_idx,
                    OpCode::new(OpCodeKind::Jump { offset: jump }, self.line()),
                );
            }
            _ => unreachable!(),
        }
    }

    fn emit_loop(&mut self, loop_start: usize) {
        let offset = self.current_chunk().borrow().len() - loop_start;
        self.emit_op_code(OpCodeKind::Loop { offset });
    }

    fn if_statement(&mut self) -> VoidResult {
        self.consume(TokenType::LeftParen, "Expected '(' after if".to_owned())?;
        self.expression()?;
        self.consume(
            TokenType::RightParen,
            "Expected ')' after if condition".to_owned(),
        )?;

        let then_jump = self.emit_jump(OpCodeKind::JumpIfFalse { offset: 0 });
        self.emit_op_code(OpCodeKind::Pop);

        self.statement()?;
        let else_jump = self.emit_jump(OpCodeKind::Jump { offset: 0 });

        self.patch_jump(then_jump);

        self.emit_op_code(OpCodeKind::Pop);
        if self.matches(&TokenType::ELSE)? {
            self.statement()?;
        }

        self.patch_jump(else_jump);
        Ok(())
    }

    fn while_statement(&mut self) -> VoidResult {
        let loop_start = self.current_chunk().borrow().len();
        self.consume(TokenType::LeftParen, "Expected '(' after while".to_owned())?;
        self.expression()?;
        self.consume(
            TokenType::RightParen,
            "Expected ')' after while condition".to_owned(),
        )?;

        let exit_jump = self.emit_jump(OpCodeKind::JumpIfFalse { offset: 0 });
        self.emit_op_code(OpCodeKind::Pop);
        self.statement()?;

        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_op_code(OpCodeKind::Pop);
        Ok(())
    }

    fn for_statement(&mut self) -> VoidResult {
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expected '(' after for".to_owned())?;

        if self.matches(&TokenType::SEMICOLON)? {
        } else if self.matches(&TokenType::VAR)? {
            self.var_statement()?;
        } else {
            self.expr_statement()?;
        }

        let mut loop_start = self.current_chunk().borrow().len();
        let mut exit_jump: Option<usize> = None;

        if !self.matches(&TokenType::SEMICOLON)? {
            self.expression()?;
            self.consume(
                TokenType::SEMICOLON,
                "Expected ';' after for condition".to_owned(),
            )?;

            exit_jump = Some(self.emit_jump(OpCodeKind::JumpIfFalse { offset: 0 }));
            self.emit_op_code(OpCodeKind::Pop);
        }

        if !self.matches(&TokenType::RightParen)? {
            let body_jump = self.emit_jump(OpCodeKind::Jump { offset: 0 });
            let increment_start = self.current_chunk().borrow().len();

            self.expression()?;
            self.emit_op_code(OpCodeKind::Pop);
            self.consume(
                TokenType::RightParen,
                "Expected ')' after for clauses".to_owned(),
            )?;

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement()?;
        self.emit_loop(loop_start);

        if let Some(jump) = exit_jump {
            self.patch_jump(jump);
            self.emit_op_code(OpCodeKind::Pop);
        }

        self.end_scope();
        Ok(())
    }

    fn start_nesting(&mut self, function_type: FunctionType, function_name: Literal) {
        self.states
            .push(CompilerState::new(function_type, function_name));
    }

    fn end_nesting(&mut self) -> CompilerState {
        self.states.pop().unwrap()
    }

    fn in_nested<F>(
        &mut self,
        function_type: FunctionType,
        function_name: Literal,
        f: F,
    ) -> Result<CompilerState, Error>
    where
        F: Fn(&mut Self) -> VoidResult,
    {
        self.start_nesting(function_type, function_name);
        f(self)?;
        Ok(self.end_nesting())
    }

    fn func_decl(&mut self) -> VoidResult {
        let global = self.parse_variable_name("Expected function name".to_owned())?;
        if self.is_local_scope() {
            self.last_local().unwrap().mark_initialized();
        }
        self.function(FunctionType::Function)?;
        self.define_variable(global);
        Ok(())
    }

    fn function(&mut self, function_type: FunctionType) -> VoidResult {
        debug!("Called function");
        let function_name = self
            .previous()
            .expect("Expected function to be called after func_decl")
            .literal
            .clone()
            .expect("Expected literal");

        let compiled_function = self
            .in_nested(function_type, function_name.clone(), |compiler| {
                compiler.begin_scope();
                compiler.consume(
                    TokenType::LeftParen,
                    "Expect '(' after function name.".to_owned(),
                )?;

                if !compiler.check(&TokenType::RightParen) {
                    loop {
                        compiler.func().borrow_mut().incr_arity();
                        let constant =
                            compiler.parse_variable_name("Expected parameter name".to_owned())?;
                        compiler.define_variable(constant);

                        if !compiler.matches(&TokenType::COMMA)? {
                            break;
                        }
                    }
                }
                compiler.consume(
                    TokenType::RightParen,
                    "Expect ')' after function parameters.".to_owned(),
                )?;
                compiler.consume(
                    TokenType::LeftBrace,
                    "Expect '{' before function body.".to_owned(),
                )?;
                compiler.block()?;
                compiler.end_scope();
                Ok(())
            })?
            .function;
        debug!("Function {function_name} compiled");
        self.emit_const(rc_refcell!(Value::Object(compiled_function)));
        debug!("Function processed");
        Ok(())
    }

    fn call(&mut self, _can_assign: bool) -> VoidResult {
        let arg_count = self.argument_list()?;
        self.emit_op_code(OpCodeKind::Call { arg_count });
        Ok(())
    }

    fn argument_list(&mut self) -> Result<usize, Error> {
        let mut arg_count = 0;
        if !self.check(&TokenType::RightParen) {
            loop {
                self.expression()?;
                arg_count += 1;
                if !self.matches(&TokenType::COMMA)? {
                    break;
                }
            }
        }
        self.consume(
            TokenType::RightParen,
            "Expect ')' after arguments list".to_owned(),
        )?;
        Ok(arg_count)
    }

    fn expression(&mut self) -> VoidResult {
        debug!("Called expression(), {}", self.debug_string());
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
        debug!("Called number() for {value}");
        self.emit_const(rc_refcell!(value));
        Ok(())
    }

    #[allow(unused_variables)]
    fn literal(&mut self, can_assign: bool) -> VoidResult {
        debug!("Called literal()");
        self.emit_op_code(match self.previous().unwrap().token_type {
            TokenType::NIL => OpCodeKind::Null,
            TokenType::FALSE => OpCodeKind::False,
            TokenType::TRUE => OpCodeKind::True,
            _ => unreachable!(),
        });
        Ok(())
    }

    fn return_stmt(&mut self) -> VoidResult {
        if matches!(self.state().function_type, FunctionType::Script) {
            return Err(self.error_at_current("Can't return from top-level code.".to_string()));
        }

        if self.matches(&TokenType::SEMICOLON)? {
            self.emit_op_code(OpCodeKind::Null);
            self.emit_op_code(OpCodeKind::Return);
        } else {
            self.expression()?;
            self.consume(
                TokenType::SEMICOLON,
                "Expected ';' after return value".to_owned(),
            )?;
            self.emit_op_code(OpCodeKind::Return);
        }

        Ok(())
    }

    #[allow(unused_variables)]
    fn string(&mut self, can_assign: bool) -> VoidResult {
        debug!("Called string()");
        self.emit_const(rc_refcell!(Value::Object(rc_refcell!(StringObject::new(
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
        debug!("Called unary for op {:?}, {}", op_type, self.debug_string());

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

    fn get_rule(&self, token_type: &TokenType) -> ParseRule<'scanner> {
        self.rules[*token_type as usize].clone()
    }

    #[allow(unused_variables)]
    fn binary(&mut self, can_assign: bool) -> VoidResult {
        let op_type = &self.previous().unwrap().token_type.clone();
        let rule = self.get_rule(op_type);
        let next_precedence = self.next_precedence(rule.precedence);

        debug!(
            "Called binary {:?}, {}, next precedence = {:?}",
            op_type,
            self.debug_string(),
            next_precedence
        );

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

    #[allow(unused_variables)]
    fn and(&mut self, can_assign: bool) -> VoidResult {
        let end_jump = self.emit_jump(OpCodeKind::JumpIfFalse { offset: 0 });
        self.emit_op_code(OpCodeKind::Pop);

        self.parse_precedence(And)?;

        self.patch_jump(end_jump);
        Ok(())
    }

    #[allow(unused_variables)]
    fn or(&mut self, can_assign: bool) -> VoidResult {
        let else_jump = self.emit_jump(OpCodeKind::JumpIfFalse { offset: 0 });
        let end_jump = self.emit_jump(OpCodeKind::Jump { offset: 0 });

        self.patch_jump(else_jump);
        self.emit_op_code(OpCodeKind::Pop);

        self.parse_precedence(Or)?;
        self.patch_jump(end_jump);
        Ok(())
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> VoidResult {
        debug!(
            "Called parse_precedence() with precedence = {:?}, {}",
            precedence,
            self.debug_string(),
        );

        self.advance()?;
        let Some(prefix_rule) = self.get_rule(&self.previous().unwrap().token_type).prefix else {
            return Err(self.error("Expected expression".to_owned()));
        };

        let can_assign = precedence as usize <= Assignment as usize;
        prefix_rule(self, can_assign)?;

        let current_token_precedence = self
            .get_rule(&self.current().unwrap().token_type)
            .precedence as usize;

        if precedence as usize > current_token_precedence {
            debug!(
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
            debug!(
                "Inside infix rule loop, precedence: {:?}({}), current precedence: {:?}({}), {}",
                precedence,
                precedence as usize,
                self.get_rule(&self.current().unwrap().token_type)
                    .precedence,
                self.get_rule(&self.current().unwrap().token_type)
                    .precedence as usize,
                self.debug_string()
            );

            self.advance()?;
            let Some(infix_rule) = self.get_rule(&self.previous().unwrap().token_type).infix else {
                continue;
            };

            debug!("Calling infix rule for {}", self.previous().unwrap());
            infix_rule(self, can_assign)?;
        };

        if can_assign && self.matches(&TokenType::EQUAL)? {
            Err(self.error("Invalid assignment target".to_owned()))
        } else {
            Ok(())
        }
    }

    fn build_rules() -> Rules<'scanner> {
        [
            /* TOKEN_LEFT_PAREN */
            ParseRule {
                prefix: Some(Compiler::grouping),
                infix: Some(Compiler::call),
                precedence: Call,
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
                infix: Some(Compiler::and),
                precedence: And,
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
                infix: Some(Compiler::or),
                precedence: Or,
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
        ]
    }
}
