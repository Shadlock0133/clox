use crate::{
    chunk::{Chunk, Opcode},
    common::DEBUG_TRACE_EXECUTION,
    compiler::{compile, CompileError},
    debug::disassembly_instruction,
    table::Table,
    value::{self, print_value, values_equal, Value},
};

pub const STACK_MAX: usize = 256;

// todo: string interning
#[derive(Default)]
pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: Stack,
    globals: Table,
}

struct Stack {
    storage: [Value; STACK_MAX],
    top: usize,
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            storage: [value::NIL; 256],
            top: Default::default(),
        }
    }
}

impl Stack {
    fn reset(&mut self) {
        self.top = 0;
    }

    fn push(&mut self, value: Value) {
        self.storage[self.top] = value;
        self.top += 1;
    }

    fn pop(&mut self) -> Value {
        self.top -= 1;
        std::mem::replace(&mut self.storage[self.top], Value::Nil)
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.storage[self.top - distance - 1]
    }
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

impl Vm {
    pub fn interpret(&mut self, source: &str) -> Result<(), Error> {
        let mut chunk = Chunk::default();
        compile(source, &mut chunk)?;
        self.chunk = chunk;
        self.ip = 0;
        self.run()
    }

    fn reset_stack(&mut self) {
        self.stack.reset()
    }

    fn runtime_error(&mut self, message: &str) {
        eprintln!("{message}");
        let line = self.chunk.get_line(self.ip - 1);
        eprintln!("[line {line}] in script");
        self.reset_stack();
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop()
    }

    fn peek(&self, distance: usize) -> &Value {
        self.stack.peek(distance)
    }

    fn read_byte(&mut self) -> u8 {
        self.chunk.code()[read_and_inc(&mut self.ip)]
    }

    fn read_constant(&mut self) -> &Value {
        let id = self.read_byte();
        self.chunk.get_constant(id)
    }

    fn read_string(&mut self) -> String {
        match self.read_constant() {
            Value::String(s) => s.to_string(),
            _ => unreachable!(),
        }
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

    fn print_similar_names(&self, _name: &str) {
        return; // TODO
        fn _levenshtein_distance(_a: &str, _b: &str) -> usize {
            todo!()
        }

        // for (global_name, _) in self.globals.iter() {
        //     if _levenshtein_distance(_name, global_name) < 3 {
        //         eprintln!("did you meant: '{global_name}'?");
        //     }
        // }
    }

    fn run(&mut self) -> Result<(), Error> {
        loop {
            if DEBUG_TRACE_EXECUTION {
                print!("          ");
                for value in &self.stack.storage[..self.stack.top] {
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
                    let constant = self.read_constant().clone();
                    self.push(constant);
                }
                Some(Opcode::Nil) => self.push(Value::Nil),
                Some(Opcode::True) => self.push(Value::Bool(true)),
                Some(Opcode::False) => self.push(Value::Bool(false)),
                Some(Opcode::Pop) => {
                    self.pop();
                }
                Some(Opcode::GetLocal) => {
                    let slot = self.read_byte();
                    self.push(self.stack.storage[slot as usize].clone());
                }
                Some(Opcode::SetLocal) => {
                    let slot = self.read_byte();
                    self.stack.storage[slot as usize] =
                        self.stack.peek(0).clone();
                }
                Some(Opcode::GetGlobal) => {
                    let name = self.read_string();
                    match self.globals.get(&name) {
                        Some(value) => self.push(value.clone()),
                        None => {
                            self.runtime_error(&format!(
                                "Undefined variable '{name}'"
                            ));
                            self.print_similar_names(&name);
                            return Err(Error::Runtime);
                        }
                    }
                }
                Some(Opcode::DefineGlobal) => {
                    let name = self.read_string();
                    self.globals.set(name, self.peek(0).clone());
                    self.pop();
                }
                Some(Opcode::SetGlobal) => {
                    let name = self.read_string();
                    if let Some(value) = self.globals.get_mut(&name) {
                        *value = self.stack.peek(0).clone();
                    } else {
                        self.runtime_error(&format!(
                            "Undefined variable '{name}'"
                        ));
                        self.print_similar_names(&name);
                        return Err(Error::Runtime);
                    }
                }
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
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::String(a), Value::String(b)) => {
                            self.push(Value::string(a.into_string() + &b))
                        }
                        (Value::Number(a), Value::Number(b)) => {
                            self.push(Value::Number(a + b))
                        }
                        _ => {
                            self.runtime_error(
                                "Operands must be numbers or strings.",
                            );
                            return Err(Error::Runtime);
                        }
                    }
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
                Some(Opcode::Print) => {
                    print_value(&self.pop());
                    println!();
                }
                Some(Opcode::Return) => {
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
