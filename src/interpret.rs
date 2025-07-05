use std::rc::Rc;

use anyhow::Error;

use crate::{chunk::Chunk, compiler::Compiler, rc_refcell, vm::VirtualMachine};

pub fn interpret(source: String, debug: bool) -> Result<(), Error> {
    let chunk = rc_refcell!(Chunk::new());
    let mut compiler = Compiler::from_source(source, debug);

    if debug {
        println!("Compiling...");
    }
    compiler.compile(Rc::clone(&chunk))?;

    let mut vm = VirtualMachine::new(Rc::clone(&chunk), debug)?;
    if debug {
        println!();
    }
    vm.exec()?;

    if debug {
        println!("Result: {}", vm.stack_top().unwrap().borrow());
    }
    Ok(())
}
