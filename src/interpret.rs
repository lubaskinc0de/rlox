use log::debug;
use std::rc::Rc;

use anyhow::Error;

use crate::{alias::StoredChunk, compiler::Compiler, vm::VirtualMachine};

pub fn interpret(source: String, chunk: StoredChunk, vm: &mut VirtualMachine) -> Result<(), Error> {
    debug!("Interpreting source..");
    let mut compiler = Compiler::from_source(source);
    debug!("Begin compilation\n");
    compiler.compile(Rc::clone(&chunk))?;

    vm.exec()?;
    Ok(())
}
