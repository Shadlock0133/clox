mod chunk;
mod common;
mod compiler;
mod debug;
mod scanner;
mod value;
mod vm;

use std::{
    env, fs,
    io::{stdin, stdout, Write},
    process::ExitCode,
};

use vm::{Error, Vm};

fn repl() {
    let mut vm = Vm::default();
    loop {
        print!("> ");
        stdout().flush().unwrap();
        let line = {
            let mut buf = String::new();
            stdin().read_line(&mut buf).unwrap();
            buf
        };
        if line.trim().is_empty() {
            println!();
            return;
        }
        vm.interpret(&line);
    }
}

fn run_file(path: String) -> ExitCode {
    let source = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Couldn't read file {path}: {e}");
            return ExitCode::from(74);
        }
    };
    let mut vm = Vm::default();
    match vm.interpret(&source) {
        Ok(()) => ExitCode::SUCCESS,
        Err(Error::Compile) => ExitCode::from(65),
        Err(Error::Runtime) => ExitCode::from(70),
    }
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let arg1 = args.next();
    let more = args.next().is_some();
    match (arg1, more) {
        (None, _) => repl(),
        (Some(file), false) => return run_file(file),
        (Some(_), true) => {
            eprintln!("Usage: clox [path]");
            return ExitCode::from(64);
        }
    }
    ExitCode::SUCCESS
}
