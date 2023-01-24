use crate::{
    chunk::{Chunk, Opcode},
    common::DEBUG_TRACE_EXECUTION,
    compiler::compile,
    debug::disassembly_instruction,
    value::{print_value, Value},
};

pub const STACK_MAX: usize = 256;

pub struct Vm<'a> {
    chunk: Option<&'a Chunk>,
    ip: usize,
    stack: [Value; STACK_MAX],
    stack_top: usize,
}

#[derive(Debug)]
pub enum Error {
    Compile,
    Runtime,
}

impl<'a> Default for Vm<'a> {
    fn default() -> Self {
        Self {
            chunk: Default::default(),
            ip: Default::default(),
            stack: [Default::default(); 256],
            stack_top: Default::default(),
        }
    }
}

impl<'a> Vm<'a> {
    pub fn interpret(&mut self, source: &str) -> Result<(), Error> {
        compile(source);
        Ok(())
    }

    fn push(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top]
    }

    fn read_byte(&mut self) -> u8 {
        self.chunk.unwrap().code()[read_byte(&mut self.ip)]
    }

    fn get_constant(&mut self) -> Value {
        self.chunk.unwrap().get_constant(self.read_byte())
    }

    fn binary_op<F: FnOnce(Value, Value) -> Value>(&mut self, f: F) {
        let b = self.pop();
        let a = self.pop();
        self.push(f(a, b));
    }

    fn run(&mut self) -> Result<(), Error> {
        loop {
            if DEBUG_TRACE_EXECUTION {
                print!("          ");
                for value in &self.stack[..self.stack_top] {
                    print!("[ ");
                    print_value(*value);
                    print!(" ]");
                }
                println!();
                disassembly_instruction(self.chunk.unwrap(), self.ip);
            }
            let instruction = self.read_byte();
            match Opcode::from_u8(instruction) {
                Some(Opcode::Constant) => {
                    let constant = self.get_constant();
                    self.push(constant);
                }
                Some(Opcode::Add) => self.binary_op(|a, b| a + b),
                Some(Opcode::Subtract) => self.binary_op(|a, b| a - b),
                Some(Opcode::Multiply) => self.binary_op(|a, b| a * b),
                Some(Opcode::Divide) => self.binary_op(|a, b| a / b),
                Some(Opcode::Negate) => {
                    let value = -self.pop();
                    self.push(value);
                }
                Some(Opcode::Return) => {
                    print_value(self.pop());
                    println!();
                    return Ok(());
                }
                _ => {
                    println!("unknown instruction: {instruction}");
                    return Err(Error::Runtime);
                }
            }
        }
    }
}

fn read_byte(value: &mut usize) -> usize {
    let ret = *value;
    *value += 1;
    ret
}
