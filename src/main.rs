#![feature(breakpoint)]
use std::{
    fs::File,
    io::{self, Read},
};

use crate::chunk::OpCode;

mod alias;
mod bin_op;
mod chunk;
mod compiler;
mod errors;
mod interpret;
mod macros;
mod parser;
mod scanner;
mod token;
mod value;
mod vm;

use crate::interpret::interpret;
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
}

fn read_file_to_string(file_name: &str) -> String {
    let mut buf = String::new();
    File::open(file_name)
        .expect("File not found")
        .read_to_string(&mut buf)
        .unwrap();
    buf
}

fn repl(debug: bool) -> Result<(), Error> {
    println!("Running RLox, mode: REPL, author: lubaskinc0de, current version: {VERSION}");
    println!("Enter program code:");
    loop {
        eprint!("> ");
        let mut prompt = String::new();
        io::stdin()
            .read_line(&mut prompt)
            .expect("Failed to read input");
        interpret(prompt, debug)?;
    }
}

fn run_source(content: String, debug: bool) -> Result<(), Error> {
    interpret(content, debug)
}

fn main() {
    let cli = CliArgs::parse();
    let file_name = cli.file_name;
    let debug = true;

    let result = match (cli.repl, file_name) {
        (true, None) => repl(debug),
        (false, None) => panic!("Pass the file name or run in REPL mode"),
        (false, Some(filename)) | (true, Some(filename)) => {
            let content = read_file_to_string(&filename);
            run_source(content, debug)
        }
    };

    match result {
        Ok(()) => return,
        Err(err) => println!("{err}"),
    }
}
