use anyhow::Error;

use crate::{
    chunk::Chunk, compiler::Compiler, errors::CompileError, rc_refcell, vm::VirtualMachine,
};

pub fn interpret(source: String, debug: bool) -> Result<(), Error> {
    let chunk = rc_refcell!(Chunk::new());
    let mut compiler = Compiler::from_source(source);

    if !compiler.compile(chunk.clone()) {
        return Err(CompileError {}.into());
    }

    let mut vm = VirtualMachine::new(chunk.clone(), debug)?;
    vm.exec()?;

    if debug {
        println!("Result: {}", vm.stack_top().unwrap().borrow());
    }
    Ok(())
}
