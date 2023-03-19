// mod thin_string;

use std::{mem::size_of, ops::Deref};

// pub use self::thin_string::ThinString;

// todo: nan-boxing
#[derive(PartialEq, Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(ThinString),
}

impl Value {
    pub fn string(v: String) -> Self {
        Value::String(ThinString::new(v))
    }
}

// if/when implementing nan-boxing, update to just sizeof(u64/f64)
const _: () = assert!(size_of::<Value>() == 2 * size_of::<u64>());

pub const NIL: Value = Value::Nil;

pub fn print_value(value: &Value) {
    match value {
        Value::Nil => print!("nil"),
        Value::Bool(b) => print!("{b}"),
        Value::Number(n) => print!("{n}"),
        Value::String(s) => print!("{}", s.as_str()),
    }
}

pub fn values_equal(a: Value, b: Value) -> bool {
    match (a, b) {
        (Value::Nil, Value::Nil) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        _ => false,
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ThinString(Box<(String, u32)>);

const _: () = assert!(size_of::<ThinString>() == size_of::<usize>());

impl ThinString {
    pub fn new(s: String) -> Self {
        let hash = hash(s.as_bytes());
        Self(Box::new((s, hash)))
    }

    pub fn hash(&self) -> u32 {
        self.0 .1
    }

    pub fn into_string(self) -> String {
        self.0 .0
    }
}

impl Deref for ThinString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0 .0
    }
}

pub fn hash(data: &[u8]) -> u32 {
    let mut hash: u32 = 2166136261;
    for &byte in data {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(16777619);
    }
    hash
}
