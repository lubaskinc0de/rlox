use log::debug;
use std::rc::Rc;

use anyhow::Error;

use crate::{
    alias::StoredChunk,
    compiler::Compiler,
    rc_refcell,
    scanner::Scanner,
    value::Value,
    vm::{CallFrame, VirtualMachine},
};

pub fn interpret(source: String, chunk: StoredChunk, vm: &mut VirtualMachine) -> Result<(), Error> {
    debug!("Interpreting source..");
    let mut scanner = Scanner::new(Rc::new(source));
    let mut compiler = Compiler::new(&mut scanner, chunk);

    debug!("Begin compilation\n");
    let function = compiler.compile()?;
    let script = rc_refcell!(Value::Object(function.clone()));

    vm.push_stored_value(script);
    vm.add_frame(CallFrame::new(function, 0, 1))?;
    vm.exec()?;

    Ok(())
}
