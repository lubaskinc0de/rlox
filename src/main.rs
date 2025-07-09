use std::{
    fs::File,
    io::{self, Read},
};

mod alias;
mod bin_op;
mod chunk;
mod compiler;
mod errors;
mod interpret;
mod macros;
mod namespace;
mod object;
mod parser;
mod scanner;
mod token;
mod value;
mod vm;

use crate::{chunk::Chunk, interpret::interpret, namespace::NameSpace, vm::VirtualMachine};
use anyhow::Error;
use clap::Parser as CliParser;

const VERSION: &str = "0.0.1";

#[derive(CliParser, Debug)]
#[command(
    version,
    about = "RLox language 2.0",
    long_about = "lubaskinc0de's Lox language implementation on Rust version 2.0"
)]
struct CliArgs {
    #[arg(short, long, default_value_t = true)]
    repl: bool,
    file_name: Option<String>,
    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

fn read_file_to_string(file_name: &str) -> String {
    let mut buf = String::new();
    File::open(file_name)
        .expect("File not found")
        .read_to_string(&mut buf)
        .unwrap();
    buf
}

fn repl(debug: bool) {
    println!("Running RLox, mode: REPL, author: lubaskinc0de, current version: {VERSION}");
    println!("Enter program code:");

    let mut globals = NameSpace::new();
    let chunk = rc_refcell!(Chunk::new());
    let mut vm = VirtualMachine::new(chunk.clone(), &mut globals, debug);
    loop {
        eprint!("> ");
        let mut prompt = String::new();
        io::stdin()
            .read_line(&mut prompt)
            .expect("Failed to read input");
        if let Err(e) = interpret(prompt, chunk.clone(), &mut vm, debug) {
            println!("{e}")
        };
    }
}

fn run_source(content: String, debug: bool) -> Result<(), Error> {
    let mut globals = NameSpace::new();
    let chunk = rc_refcell!(Chunk::new());
    let mut vm = VirtualMachine::new(chunk.clone(), &mut globals, debug);
    interpret(content, chunk, &mut vm, debug)
}

fn main() {
    let cli = CliArgs::parse();
    let file_name = cli.file_name;
    let debug = cli.debug;

    let result = match (cli.repl, file_name) {
        (true, None) => {
            repl(debug);
            Ok(())
        }
        (false, None) => panic!("Pass the file name or run in REPL mode"),
        (false, Some(filename)) | (true, Some(filename)) => {
            let content = read_file_to_string(&filename);
            run_source(content, debug)
        }
    };

    match result {
        Ok(()) => (),
        Err(err) => println!("{err}"),
    }
}
