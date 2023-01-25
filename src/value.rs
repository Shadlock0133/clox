#[derive(PartialEq)]
enum Value {
    Nil,
    Bool(bool),
    Number(f64),
}

pub const NIL: Value = Value::Nil;

pub fn print_value(value: Value) {
    match value {
        Value::Nil => print!("nil"),
        Value::Bool(b) => print!("{b}"),
        Value::Number(n) => print!("{n}"),
    }
}
