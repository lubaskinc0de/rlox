use anyhow::Error;

use crate::{
    chunk::Chunk, compiler::Compiler, errors::CompileError, parser::Parser, rc_refcell, scanner::Scanner, vm::VirtualMachine
};

pub fn interpret(source: String) -> Result<(), Error> {
    let chunk = rc_refcell!(Chunk::new());
    let scanner = Scanner::new(source);
    let parser = Parser::new();
    let mut compiler = Compiler::new(parser, scanner);

    if !compiler.compile(chunk.clone()) {
        return Err(CompileError {}.into());
    }

    let mut vm = VirtualMachine::new(chunk.clone(), true)?;
    vm.exec()
}
