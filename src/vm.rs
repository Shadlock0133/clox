use crate::{
    chunk::{Chunk, Opcode},
    common::DEBUG_TRACE_EXECUTION,
    compiler::{compile, CompileError},
    debug::disassembly_instruction,
    value::{self, print_value, values_equal, Value},
};

pub const STACK_MAX: usize = 256;

pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: [Value; STACK_MAX],
    stack_top: usize,
}

#[derive(Debug)]
pub enum Error {
    Compile(CompileError),
    Runtime,
}

impl From<CompileError> for Error {
    fn from(value: CompileError) -> Self {
        Self::Compile(value)
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self {
            chunk: Default::default(),
            ip: Default::default(),
            stack: [value::NIL; 256],
            stack_top: Default::default(),
        }
    }
}

impl Vm {
    pub fn interpret(&mut self, source: &str) -> Result<(), Error> {
        let mut chunk = Chunk::default();
        compile(source, &mut chunk)?;
        self.chunk = chunk;
        self.ip = 0;
        self.run()
    }

    fn reset_stack(&mut self) {
        self.stack_top = 0;
    }

    fn runtime_error(&mut self, message: &str) {
        eprintln!("{message}");
        let line = self.chunk.get_line(self.ip - 1);
        eprintln!("[line {line}] in script");
        self.reset_stack();
    }

    fn push(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        std::mem::replace(&mut self.stack[self.stack_top], Value::Nil)
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack_top - distance]
    }

    fn read_byte(&mut self) -> u8 {
        self.chunk.code()[read_and_inc(&mut self.ip)]
    }

    fn get_constant(&mut self) -> &Value {
        let id = self.read_byte();
        self.chunk.get_constant(id)
    }

    fn binary_op<F: FnOnce(f64, f64) -> Value>(
        &mut self,
        f: F,
    ) -> Result<(), Error> {
        let b = self.pop();
        let a = self.pop();
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => self.push(f(a, b)),
            _ => {
                self.runtime_error("Operands must be numbers.");
                return Err(Error::Runtime);
            }
        }
        Ok(())
    }

    fn run(&mut self) -> Result<(), Error> {
        loop {
            if DEBUG_TRACE_EXECUTION {
                print!("          ");
                for value in &self.stack[..self.stack_top] {
                    print!("[ ");
                    print_value(value);
                    print!(" ]");
                }
                println!();
                disassembly_instruction(&self.chunk, self.ip);
            }
            let instruction = self.read_byte();
            match Opcode::from_u8(instruction) {
                Some(Opcode::Constant) => {
                    let constant = self.get_constant().clone();
                    self.push(constant);
                }
                Some(Opcode::Nil) => self.push(Value::Nil),
                Some(Opcode::True) => self.push(Value::Bool(true)),
                Some(Opcode::False) => self.push(Value::Bool(false)),
                Some(Opcode::Equal) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(values_equal(a, b)));
                }
                Some(Opcode::Greater) => {
                    self.binary_op(|a, b| Value::Bool(a > b))?
                }
                Some(Opcode::Less) => {
                    self.binary_op(|a, b| Value::Bool(a < b))?
                }
                Some(Opcode::Add) => {
                    self.binary_op(|a, b| Value::Number(a + b))?
                }
                Some(Opcode::Subtract) => {
                    self.binary_op(|a, b| Value::Number(a - b))?
                }
                Some(Opcode::Multiply) => {
                    self.binary_op(|a, b| Value::Number(a * b))?
                }
                Some(Opcode::Divide) => {
                    self.binary_op(|a, b| Value::Number(a / b))?
                }
                Some(Opcode::Negate) => {
                    let value = self.pop();
                    if let Value::Number(n) = value {
                        self.push(Value::Number(-n));
                    } else {
                        self.runtime_error("Operand must be a number.");
                        return Err(Error::Runtime);
                    }
                }
                Some(Opcode::Not) => {
                    let value = self.pop();
                    self.push(Value::Bool(is_falsey(value)));
                }
                Some(Opcode::Return) => {
                    print_value(&self.pop());
                    println!();
                    return Ok(());
                }
                None => {
                    println!("unknown instruction: {instruction}");
                    return Err(Error::Runtime);
                }
            }
        }
    }
}

fn read_and_inc(value: &mut usize) -> usize {
    let ret = *value;
    *value += 1;
    ret
}

fn is_falsey(value: Value) -> bool {
    match value {
        Value::Nil | Value::Bool(false) => true,
        _ => false,
    }
}
