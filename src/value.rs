#[derive(PartialEq, Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
}

pub const NIL: Value = Value::Nil;

pub fn print_value(value: &Value) {
    match value {
        Value::Nil => print!("nil"),
        Value::Bool(b) => print!("{b}"),
        Value::Number(n) => print!("{n}"),
    }
}

pub fn values_equal(a: Value, b: Value) -> bool {
    match (a, b) {
        (Value::Nil, Value::Nil) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a == b,
        _ => false,
    }
}
