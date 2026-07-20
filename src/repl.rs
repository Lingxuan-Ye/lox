use crate::interpreter::Interpreter;
use core::slice;
use std::io;
use std::io::{BufRead, BufWriter, StdinLock, StdoutLock};

#[derive(Debug)]
pub struct Repl {
    input_pool: Vec<LeakedInput>,
    stdin: StdinLock<'static>,
    interpreter: Interpreter<'static, BufWriter<StdoutLock<'static>>>,
}

#[derive(Debug)]
struct LeakedInput {
    ptr: *mut u8,
    len: usize,
    cap: usize,
}

impl Repl {
    pub fn new() -> Self {
        let input_pool = Vec::new();
        let stdin = io::stdin().lock();
        let stdout = io::stdout().lock();
        let output = BufWriter::new(stdout);
        let interpreter = Interpreter::new(output);
        Self {
            input_pool,
            stdin,
            interpreter,
        }
    }

    pub fn run(&mut self) {
        loop {
            let mut input = String::new();
            let mut last_line_blank = false;
            loop {
                match self.stdin.read_line(&mut input) {
                    Err(error) => panic!("{error:?}"),
                    Ok(0) => return,
                    Ok(1) => {
                        if last_line_blank {
                            break;
                        } else {
                            last_line_blank = true;
                        }
                    }
                    Ok(_) => (),
                }
            }
            let (ptr, len, cap) = input.into_raw_parts();
            let leaked = LeakedInput { ptr, len, cap };
            let source = unsafe {
                let slice = slice::from_raw_parts(ptr, len);
                str::from_utf8_unchecked(slice)
            };
            self.input_pool.push(leaked);
            self.interpreter.interpret(source).unwrap();
            self.interpreter.flush().unwrap();
        }
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Repl {
    fn drop(&mut self) {
        for leaked in &self.input_pool {
            unsafe {
                String::from_raw_parts(leaked.ptr, leaked.len, leaked.cap);
            }
        }
    }
}
