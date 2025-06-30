use std::rc::Rc;

use crate::{chunk::{Chunk, OpCode}, value::Value, vm::VirtualMachine};

mod chunk;
mod macros;
mod value;
mod vm;
mod errors;


fn main() {
    let mut chunk = Chunk::new();
    let value = Value::Float(32.5);
    let const_idx = chunk.push_const(Rc::new(value));
    chunk.push(OpCode::OpConst { const_idx: const_idx, line: 0 });

    let mut vm = VirtualMachine::new(Rc::new(chunk), true).unwrap();
    vm.exec_chunk().unwrap();
}
