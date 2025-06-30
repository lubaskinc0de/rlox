use std::rc::Rc;

use crate::{
    chunk::{Chunk, OpCode},
    value::Value,
    vm::VirtualMachine,
};

mod chunk;
mod errors;
mod macros;
mod value;
mod vm;
mod alias;

fn main() {
    let mut chunk = Chunk::new();
    let value = Value::Float(32.5);
    let const_idx = chunk.push_const(rc_refcell!(value));

    chunk.push(OpCode::OpConst {
        const_idx: const_idx,
        line: 0,
    });

    chunk.push(OpCode::OpNegate { line: 1 });

    let mut vm = VirtualMachine::new(chunk, true).unwrap();
    vm.exec().unwrap();
    println!("{:?}", vm.stack_top());
}
