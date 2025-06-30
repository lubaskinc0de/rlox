use std::{fs::File, io::{self, Read}};

use crate::chunk::OpCode;

mod alias;
mod bin_op;
mod chunk;
mod errors;
mod macros;
mod value;
mod vm;
mod scanner;
mod token;

use anyhow::Error;
use clap::Parser as CliParser;

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

fn repl() -> () {
    loop {
        print!("> ");
        let mut prompt = String::new();
        io::stdin().read_line(&mut prompt).expect("Failed to read input");
        
    }
}

fn run_source(content: String) -> Result<(), Error> {
    Ok(())
}

fn main() {
    let cli = CliArgs::parse();
    let file_name = cli.file_name;

    match (cli.repl, file_name) {
        (true, Some(_)) => panic!("You must choose either to run in REPL mode or to pass the file name, but not both."),
        (true, None) => {
            repl()
        },
        (false, None) => panic!("Pass the file name or run in REPL mode"),
        (false, Some(filename)) => {
            let content = read_file_to_string(&filename);
            run_source(content).unwrap()
        },
    }
}
