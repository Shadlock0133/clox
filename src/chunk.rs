use crate::{debug::disassembly_instruction, value::Value};

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

pub type Id = u8;

impl Chunk {
    pub fn write_byte(&mut self, byte: u8, line: u32) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn disassembly(&self, name: &str) {
        println!("== {name} ==");

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

    pub fn get_constant(&self, id: Id) -> Value {
        self.constants[usize::from(id)]
    }

    pub fn get_line_if_first(&self, offset: usize) -> Option<u32> {
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            None
        } else {
            Some(self.lines[offset])
        }
    }
}
