use std::rc::Rc;

use anyhow::Error;

use crate::{alias::StoredChunk, compiler::Compiler, vm::VirtualMachine};

pub fn interpret(source: String, chunk: StoredChunk, vm: &mut VirtualMachine, debug: bool) -> Result<(), Error> {
    let mut compiler = Compiler::from_source(source, debug);

    if debug {
        println!("Compiling...");
    }
    compiler.compile(Rc::clone(&chunk))?;

    if debug {
        println!();
    }

    vm.exec()?;
    Ok(())
}
