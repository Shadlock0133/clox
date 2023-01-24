macro_rules! opcode {
    ( $(#[$attr:meta])* $vis:vis enum $name:ident : $type:ty {
        $($variant:ident),* $(,)?
    } ) => {
        #[repr($type)]
        $(#[$attr])*
        $vis enum $name {
            $($variant,)*
        }

        #[allow(non_upper_case_globals)]
        mod consts {
            $(pub const $variant: $type = super::$name::$variant as $type;)*
        }

        impl $name {
            pub fn as_u8(self) -> $type {
                self as _
            }

            pub fn from_u8(value: $type) -> Option<Self> {
                match value {
                    $(consts::$variant => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }
    };
}

opcode! {
    #[derive(Clone, Copy)]
    pub enum Opcode: u8 {
        Return,
        Add,
        Subtract,
        Multiply,
        Divide,
        Negate,
        Constant,
    }
}

#[derive(Default)]
pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<u32>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn write_byte(&mut self, byte: u8, line: u32) {
        self.code.push(byte);
        self.lines.push(line);
    }

    fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn disassembly(&self, name: &str) {
        eprintln!("== {name} ==");

        let mut offset = 0;
        while offset < self.code.len() {
            offset = disassembly_instruction(&self, offset);
        }
    }

    pub fn add_constant(&mut self, value: Value) -> Id {
        let id = self.constants.len();
        self.constants.push(value);
        id.try_into().unwrap()
    }

    fn get_constant(&self, id: Id) -> Value {
        self.constants[usize::from(id)]
    }

    fn get_line_if_first(&self, offset: usize) -> Option<u32> {
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            None
        } else {
            Some(self.lines[offset])
        }
    }
}

fn disassembly_instruction(chunk: &Chunk, offset: usize) -> usize {
    eprint!("{offset:04}: ");
    match chunk.get_line_if_first(offset) {
        Some(line) => eprint!("{:4} ", line),
        None => eprint!("   | "),
    }
    let op = chunk.code()[offset];
    let size = match Opcode::from_u8(op) {
        Some(Opcode::Constant) => {
            constant_instruction("OP_CONSTANT", chunk, offset)
        }
        Some(Opcode::Add) => simple_instruction("OP_ADD", offset),
        Some(Opcode::Subtract) => simple_instruction("OP_SUBTRACT", offset),
        Some(Opcode::Multiply) => simple_instruction("OP_MULTIPLY", offset),
        Some(Opcode::Divide) => simple_instruction("OP_DIVIDE", offset),
        Some(Opcode::Negate) => simple_instruction("OP_NEGATE", offset),
        Some(Opcode::Return) => simple_instruction("OP_RETURN", offset),
        _ => {
            eprintln!("unknown opcode: {op}");
            1
        }
    };
    offset + size
}

fn simple_instruction(name: &str, _offset: usize) -> usize {
    eprintln!("{name}");
    1
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.code()[offset + 1];
    eprint!("{name:16} {constant:4} '");
    print_value(chunk.get_constant(constant.into()));
    eprintln!("'");
    2
}

fn print_value(value: Value) {
    eprint!("{value}")
}

type Value = f64;
type Id = u8;

struct Vm<'a> {
    chunk: Option<&'a Chunk>,
    ip: usize,
    stack: [Value; STACK_MAX],
    stack_top: usize,
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
    fn interpret(&mut self, chunk: &'a Chunk) -> Result<(), Error> {
        self.chunk = Some(chunk);
        self.ip = 0;
        self.run()
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
                eprint!("          ");
                for value in &self.stack[..self.stack_top] {
                    eprint!("[ ");
                    print_value(*value);
                    eprint!(" ]");
                }
                eprintln!();
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
                    eprintln!();
                    return Ok(());
                }
                _ => {
                    eprintln!("unknown instruction: {instruction}");
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

#[derive(Debug)]
enum Error {
    Compile,
    Runtime,
}

const DEBUG_TRACE_EXECUTION: bool = true;
const STACK_MAX: usize = 256;

fn main() {
    let mut chunk = Chunk::default();
    let constant = chunk.add_constant(1.2);
    chunk.write_byte(Opcode::Constant.as_u8(), 123);
    chunk.write_byte(constant, 123);

    let constant = chunk.add_constant(3.4);
    chunk.write_byte(Opcode::Constant.as_u8(), 123);
    chunk.write_byte(constant, 123);

    chunk.write_byte(Opcode::Add.as_u8(), 123);

    let constant = chunk.add_constant(5.6);
    chunk.write_byte(Opcode::Constant.as_u8(), 123);
    chunk.write_byte(constant, 123);

    chunk.write_byte(Opcode::Divide.as_u8(), 123);
    chunk.write_byte(Opcode::Negate.as_u8(), 123);
    chunk.write_byte(Opcode::Return.as_u8(), 123);
    chunk.disassembly("test chunk");
    let mut vm = Vm::default();
    vm.interpret(&chunk).unwrap();
}
