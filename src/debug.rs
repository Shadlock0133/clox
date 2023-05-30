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
        Some(Opcode::Pop) => simple_instruction("OP_POP", offset),
        Some(Opcode::GetLocal) => {
            byte_instruction("OP_GET_LOCAL", chunk, offset)
        }
        Some(Opcode::SetLocal) => {
            byte_instruction("OP_SET_LOCAL", chunk, offset)
        }
        Some(Opcode::GetGlobal) => {
            constant_instruction("OP_GET_GLOBAL", chunk, offset)
        }
        Some(Opcode::DefineGlobal) => {
            constant_instruction("OP_DEFINE_GLOBAL", chunk, offset)
        }
        Some(Opcode::SetGlobal) => {
            constant_instruction("OP_SET_GLOBAL", chunk, offset)
        }
        Some(Opcode::Equal) => simple_instruction("OP_EQUAL", offset),
        Some(Opcode::Greater) => simple_instruction("OP_GREATER", offset),
        Some(Opcode::Less) => simple_instruction("OP_LESS", offset),
        Some(Opcode::Add) => simple_instruction("OP_ADD", offset),
        Some(Opcode::Subtract) => simple_instruction("OP_SUBTRACT", offset),
        Some(Opcode::Multiply) => simple_instruction("OP_MULTIPLY", offset),
        Some(Opcode::Divide) => simple_instruction("OP_DIVIDE", offset),
        Some(Opcode::Not) => simple_instruction("OP_NOT", offset),
        Some(Opcode::Negate) => simple_instruction("OP_NEGATE", offset),
        Some(Opcode::Print) => simple_instruction("OP_PRINT", offset),
        Some(Opcode::Jump) => {
            jump_instruction("OP_JUMP", chunk, JumpDirection::Forward, offset)
        }
        Some(Opcode::JumpIfFalse) => jump_instruction(
            "OP_JUMP_IF_FALSE",
            chunk,
            JumpDirection::Forward,
            offset,
        ),
        Some(Opcode::Loop) => {
            jump_instruction("OP_LOOP", chunk, JumpDirection::Backward, offset)
        }
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

fn byte_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let slot = chunk.code()[offset + 1];
    println!("{name:16} {slot:4}");
    2
}

enum JumpDirection {
    Forward,
    Backward,
}

fn jump_instruction(
    name: &str,
    chunk: &Chunk,
    dir: JumpDirection,
    offset: usize,
) -> usize {
    let bytes = chunk.code()[offset + 1..][..2].try_into().unwrap();
    let jump: usize = u16::from_le_bytes(bytes).into();
    let target = match dir {
        JumpDirection::Forward => offset + jump + 3,
        JumpDirection::Backward => offset - jump + 3,
    };
    println!("{name:16} {offset:4} -> {target}");
    3
}

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.code()[offset + 1];
    print!("{name:16} {constant:4} '");
    print_value(&chunk.get_constant(constant.into()));
    println!("'");
    2
}
