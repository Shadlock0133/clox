pub struct Scanner<'s> {
    source: &'s str,
    current: usize,
    line: u32,
}

impl<'s> Scanner<'s> {
    pub fn new(source: &'s str) -> Self {
        Self {
            source,
            current: 0,
            line: 1,
        }
    }
}
