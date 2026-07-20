use lox::interpreter::Interpreter;
use std::env;
use std::fs;
use std::io;
use std::process;

fn main() {
    let Some(path) = env::args_os().nth(1) else {
        eprintln!("usage: minilox <script>");
        process::exit(64);
    };
    let source = fs::read_to_string(path).unwrap_or_else(|error| {
        eprintln!("error: {error}");
        process::exit(65);
    });
    let output = io::stdout().lock();
    let output = io::BufWriter::new(output);
    let mut interpreter = Interpreter::new(output);
    if let Err(error) = interpreter.interpret(&source) {
        eprintln!("error: {error:?}");
        process::exit(70);
    }
}
