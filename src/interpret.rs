use anyhow::Error;

use crate::{
    chunk::Chunk, compiler::Compiler, errors::CompileError, rc_refcell, vm::VirtualMachine
};

pub fn interpret(source: String) -> Result<(), Error> {
    let chunk = rc_refcell!(Chunk::new());
    let mut compiler = Compiler::from_source(source);

    if !compiler.compile(chunk.clone()) {
        return Err(CompileError {}.into());
    }

    let mut vm = VirtualMachine::new(chunk.clone(), true)?;
    vm.exec()
}
