use std::fmt::Display;

#[derive(Debug)]
pub struct ParseError {
    msg: String,
    line: usize,
    col: usize,
}

impl ParseError {
    pub(crate) fn new(msg: String, source: &str, byte_idx: usize) -> Self {
        // Calculate line from number of newlines up to the error byte index

        let line = source[0..byte_idx]
            .as_bytes()
            .iter()
            .filter(|b| b == &&b'\n')
            .count();

        let last_newline_idx = source[0..byte_idx]
            .as_bytes()
            .iter()
            .enumerate()
            .filter(|(_, b)| b == &&b'\n')
            .last()
            .unwrap_or((0, &0))
            .0;

        let col = byte_idx - last_newline_idx;

        Self { msg, line, col }
    }

    pub fn message(&self) -> &String {
        &self.msg
    }

    pub fn line_and_column(&self) -> (usize, usize) {
        (self.line, self.col)
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (line, col) = self.line_and_column();
        write!(f, "[{}:{}] {}", line, col, self.message())
    }
}
