use crate::{
    chunk::{Chunk, Opcode},
    value::print_value,
};

pub fn disassembly_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{offset:04}: ");
    match chunk.get_line_if_first(offset) {
        Some(line) => print!("{:4} ", line),
        None => print!("   | "),
    }
    let op = chunk.code()[offset];
    let size = match Opcode::from_u8(op) {
        Some(Opcode::Constant) => {
            constant_instruction("OP_CONSTANT", chunk, offset)
        }
        Some(Opcode::Nil) => simple_instruction("OP_NIL", offset),
        Some(Opcode::True) => simple_instruction("OP_TRUE", offset),
        Some(Opcode::False) => simple_instruction("OP_FALSE", offset),
        Some(Opcode::Equal) => simple_instruction("OP_EQUAL", offset),
        Some(Opcode::Greater) => simple_instruction("OP_GREATER", offset),
        Some(Opcode::Less) => simple_instruction("OP_LESS", offset),
        Some(Opcode::Add) => simple_instruction("OP_ADD", offset),
        Some(Opcode::Subtract) => simple_instruction("OP_SUBTRACT", offset),
        Some(Opcode::Multiply) => simple_instruction("OP_MULTIPLY", offset),
        Some(Opcode::Divide) => simple_instruction("OP_DIVIDE", offset),
        Some(Opcode::Not) => simple_instruction("OP_NOT", offset),
        Some(Opcode::Negate) => simple_instruction("OP_NEGATE", offset),
        Some(Opcode::Return) => simple_instruction("OP_RETURN", offset),
        None => {
            println!("unknown opcode: {op}");
            1
        }
    };
    offset + size
}

fn simple_instruction(name: &str, _offset: usize) -> usize {
    println!("{name}");
    1
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.code()[offset + 1];
    print!("{name:16} {constant:4} '");
    print_value(&chunk.get_constant(constant.into()));
    println!("'");
    2
}
