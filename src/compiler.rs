use std::mem;

use crate::{
    chunk::{Chunk, Id, Opcode},
    common::DEBUG_PRINT_CODE,
    scanner::{Scanner, Token, TokenType},
    value::Value,
};

#[derive(Debug)]
pub struct CompileError;

struct Parser<'s, 'c> {
    scanner: Scanner<'s>,
    chunk: &'c mut Chunk,
    current: Token<'s>,
    previous: Token<'s>,
    had_error: bool,
    panic_mode: bool,
}

const EMPTY_TOKEN: Token = Token {
    r#type: TokenType::Error,
    lexeme: "",
    line: 0,
};

impl<'s, 'c> Parser<'s, 'c> {
    fn new(scanner: Scanner<'s>, chunk: &'c mut Chunk) -> Self {
        Self {
            scanner,
            chunk,
            current: EMPTY_TOKEN,
            previous: EMPTY_TOKEN,
            had_error: false,
            panic_mode: false,
        }
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.current.clone(), message)
    }

    fn error(&mut self, message: &str) {
        self.error_at(self.previous.clone(), message)
    }

    fn error_at(&mut self, token: Token, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;

        eprint!("[line {}] Error", token.line);
        match token.r#type {
            TokenType::Eof => eprint!(" at end"),
            TokenType::Error => (),
            _ => eprint!(" at '{}'", token.lexeme),
        }
        eprintln!(": {message}");
        self.had_error = true;
    }

    fn advance(&mut self) {
        self.previous = mem::replace(&mut self.current, EMPTY_TOKEN);
        loop {
            self.current = self.scanner.scan_token();
            if self.current.r#type != TokenType::Error {
                break;
            }
            self.error_at_current(self.current.lexeme)
        }
    }

    fn check(&mut self, r#type: TokenType) -> bool {
        self.current.r#type == r#type
    }

    fn match_(&mut self, r#type: TokenType) -> bool {
        if self.check(r#type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, r#type: TokenType, message: &str) {
        if !self.match_(r#type) {
            self.error_at_current(message);
        }
    }

    fn emit_byte(&mut self, byte: u8) {
        self.chunk.write_byte(byte, self.previous.line);
    }

    fn emit_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.emit_byte(*byte);
        }
    }

    fn make_constant(&mut self, value: Value) -> Id {
        if let Some(id) = self.chunk.find_constant(&value) {
            return id;
        }
        if self.chunk.constants_len() == Id::MAX {
            self.error("Too many constants in one chunk.");
            return 0;
        }
        self.chunk.add_constant(value)
    }

    fn emit_constant(&mut self, value: Value) {
        let id = self.make_constant(value);
        self.emit_bytes(&[Opcode::Constant.as_u8(), id]);
    }

    fn emit_return(&mut self) {
        self.emit_byte(Opcode::Return.as_u8());
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        if DEBUG_PRINT_CODE {
            if !self.had_error {
                self.chunk.disassembly("code");
            }
        }
    }

    fn number(&mut self, _can_assign: bool) {
        let value = self.previous.lexeme.parse().unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn string(&mut self, _can_assign: bool) {
        let s = self
            .previous
            .lexeme
            .strip_prefix('"')
            .unwrap()
            .strip_suffix('"')
            .unwrap();
        self.emit_constant(Value::string(String::from(s)))
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let arg = self.identifier_constant(name);
        if can_assign && self.match_(TokenType::Equal) {
            self.expression();
            self.emit_bytes(&[Opcode::SetGlobal.as_u8(), arg]);
        } else {
            self.emit_bytes(&[Opcode::GetGlobal.as_u8(), arg]);
        }
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.previous.clone(), can_assign);
    }

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self, _can_assign: bool) {
        let operator = self.previous.r#type;
        self.parse_precedence(Precedence::Unary);
        match operator {
            TokenType::Bang => self.emit_byte(Opcode::Not.as_u8()),
            TokenType::Minus => self.emit_byte(Opcode::Negate.as_u8()),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self, _can_assign: bool) {
        let operator = self.previous.r#type;
        let rule = get_rule(operator);
        self.parse_precedence(rule.precedence.next());
        match operator {
            TokenType::BangEqual => {
                self.emit_bytes(&[Opcode::Equal.as_u8(), Opcode::Not.as_u8()])
            }
            TokenType::EqualEqual => self.emit_byte(Opcode::Equal.as_u8()),
            TokenType::Greater => self.emit_byte(Opcode::Greater.as_u8()),
            TokenType::GreaterEqual => {
                self.emit_bytes(&[Opcode::Less.as_u8(), Opcode::Not.as_u8()])
            }
            TokenType::Less => self.emit_byte(Opcode::Less.as_u8()),
            TokenType::LessEqual => {
                self.emit_bytes(&[Opcode::Greater.as_u8(), Opcode::Not.as_u8()])
            }
            TokenType::Plus => self.emit_byte(Opcode::Add.as_u8()),
            TokenType::Minus => self.emit_byte(Opcode::Subtract.as_u8()),
            TokenType::Star => self.emit_byte(Opcode::Multiply.as_u8()),
            TokenType::Slash => self.emit_byte(Opcode::Divide.as_u8()),
            _ => unreachable!(),
        }
    }

    fn literal(&mut self, _can_assign: bool) {
        match self.previous.r#type {
            TokenType::False => self.emit_byte(Opcode::False.as_u8()),
            TokenType::Nil => self.emit_byte(Opcode::Nil.as_u8()),
            TokenType::True => self.emit_byte(Opcode::True.as_u8()),
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = get_rule(self.previous.r#type).prefix;
        let Some(rule) = prefix_rule else {
            self.error("Expect expression.");
            return;
        };

        let can_assign = precedence <= Precedence::Assignment;
        rule(self, can_assign);

        while precedence <= get_rule(self.current.r#type).precedence {
            self.advance();
            let infix_rule = get_rule(self.previous.r#type).infix;
            infix_rule.unwrap()(self, can_assign);
        }

        if can_assign && self.match_(TokenType::Equal) {
            self.error("Invalid assignment target");
        }
    }

    fn identifier_constant(&mut self, name: Token) -> Id {
        self.make_constant(Value::string(name.lexeme.to_string()))
    }

    fn parse_variable(&mut self, error_message: &str) -> Id {
        self.consume(TokenType::Identifier, error_message);
        self.identifier_constant(self.previous.clone())
    }

    fn define_global(&mut self, global: Id) {
        self.emit_bytes(&[Opcode::DefineGlobal.as_u8(), global])
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment)
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");
        if self.match_(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(Opcode::Nil.as_u8());
        }
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );
        self.define_global(global);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(Opcode::Print.as_u8());
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;
        while !self.check(TokenType::Eof) {
            if self.previous.r#type == TokenType::Semicolon {
                return;
            }
            match self.current.r#type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => (),
            }
            self.advance();
        }
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(Opcode::Pop.as_u8());
    }

    fn declaration(&mut self) {
        if self.match_(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }
        if self.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.match_(TokenType::Print) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }
}

type ParseFn<'s, 'c> = for<'a> fn(&'a mut Parser<'s, 'c>, bool);

struct ParseRule<'s, 'c> {
    prefix: Option<ParseFn<'s, 'c>>,
    infix: Option<ParseFn<'s, 'c>>,
    precedence: Precedence,
}

fn get_rule<'s, 'c>(r#type: TokenType) -> ParseRule<'s, 'c> {
    use Parser as P;
    use Precedence as Pr;
    use TokenType as TT;

    #[rustfmt::skip]
    let (prefix, infix, precedence): (Option<ParseFn>, Option<ParseFn>, _) =
    match r#type {
        TT::LeftParen =>    (Some(P::grouping),            None, Pr::None),
        TT::RightParen =>   (             None,            None, Pr::None),
        TT::LeftBrace =>    (             None,            None, Pr::None),
        TT::RightBrace =>   (             None,            None, Pr::None),
        TT::Comma =>        (             None,            None, Pr::None),
        TT::Dot =>          (             None,            None, Pr::None),
        TT::Minus =>        (   Some(P::unary), Some(P::binary), Pr::Term),
        TT::Plus =>         (             None, Some(P::binary), Pr::Term),
        TT::Semicolon =>    (             None,            None, Pr::None),
        TT::Slash =>        (             None, Some(P::binary), Pr::Factor),
        TT::Star =>         (             None, Some(P::binary), Pr::Factor),
        TT::Bang =>         (   Some(P::unary),            None, Pr::None),
        TT::BangEqual =>    (             None, Some(P::binary), Pr::Equality),
        TT::Equal =>        (             None,            None, Pr::None),
        TT::EqualEqual =>   (             None, Some(P::binary), Pr::Equality),
        TT::Greater =>      (             None, Some(P::binary), Pr::Comparison),
        TT::GreaterEqual => (             None, Some(P::binary), Pr::Comparison),
        TT::Less =>         (             None, Some(P::binary), Pr::Comparison),
        TT::LessEqual =>    (             None, Some(P::binary), Pr::Comparison),
        TT::Identifier =>   (Some(P::variable),            None, Pr::None),
        TT::String =>       (  Some(P::string),            None, Pr::None),
        TT::Number =>       (  Some(P::number),            None, Pr::None),
        TT::And =>          (             None,            None, Pr::None),
        TT::Class =>        (             None,            None, Pr::None),
        TT::Else =>         (             None,            None, Pr::None),
        TT::False =>        ( Some(P::literal),            None, Pr::None),
        TT::For =>          (             None,            None, Pr::None),
        TT::Fun =>          (             None,            None, Pr::None),
        TT::If =>           (             None,            None, Pr::None),
        TT::Nil =>          ( Some(P::literal),            None, Pr::None),
        TT::Or =>           (             None,            None, Pr::None),
        TT::Print =>        (             None,            None, Pr::None),
        TT::Return =>       (             None,            None, Pr::None),
        TT::Super =>        (             None,            None, Pr::None),
        TT::This =>         (             None,            None, Pr::None),
        TT::True =>         ( Some(P::literal),            None, Pr::None),
        TT::Var =>          (             None,            None, Pr::None),
        TT::While =>        (             None,            None, Pr::None),
        TT::Error =>        (             None,            None, Pr::None),
        TT::Eof =>          (             None,            None, Pr::None),
    };
    ParseRule {
        prefix,
        infix,
        precedence,
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl Precedence {
    fn next(self) -> Self {
        match self {
            Self::None => Self::Assignment,
            Self::Assignment => Self::Or,
            Self::Or => Self::And,
            Self::And => Self::Equality,
            Self::Equality => Self::Comparison,
            Self::Comparison => Self::Term,
            Self::Term => Self::Factor,
            Self::Factor => Self::Unary,
            Self::Unary => Self::Call,
            Self::Call => Self::Primary,
            Self::Primary => unimplemented!(),
        }
    }
}

pub fn compile(source: &str, chunk: &mut Chunk) -> Result<(), CompileError> {
    let scanner = Scanner::new(source);
    let mut parser = Parser::new(scanner, chunk);

    parser.advance();
    while !parser.match_(TokenType::Eof) {
        parser.declaration();
    }
    parser.end_compiler();

    if parser.had_error {
        Err(CompileError)
    } else {
        Ok(())
    }
}
