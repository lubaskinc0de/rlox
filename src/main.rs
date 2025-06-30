use crate::{
    chunk::{Chunk, OpCode},
    value::Value,
    vm::VirtualMachine,
};

mod alias;
mod bin_op;
mod chunk;
mod errors;
mod macros;
mod value;
mod vm;

fn main() {
    let mut chunk = Chunk::new();
    let a = Value::Float(12.0);
    let a_idx = chunk.push_const(rc_refcell!(a));
    let b = Value::Float(6.0);
    let b_idx = chunk.push_const(rc_refcell!(b));

    chunk.push(OpCode::OpConst {
        const_idx: a_idx,
        line: 0,
    });
    chunk.push(OpCode::OpConst {
        const_idx: b_idx,
        line: 1,
    });
    chunk.push(OpCode::OpDiv { line: 2 });

    let mut vm = VirtualMachine::new(chunk, true).unwrap();
    vm.exec().unwrap();
    println!("Result of calculating 12 / 6: {:?}", vm.stack_top().unwrap().borrow());
}
