use crate::scanner::Scanner;

pub fn compile(source: &str) {
    Scanner::new(source);
}
