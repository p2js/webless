use std::fmt::Display;

#[derive(Debug)]
pub struct ParseError {
    msg: String,
    line: usize,
    col: usize,
}

impl ParseError {
    pub(crate) fn new(msg: String, source: &str, byte_idx: usize) -> Self {
        // Calculate line and column from byte index of last newline character
        let last_newline = source[0..byte_idx]
            .as_bytes()
            .iter()
            .enumerate()
            .filter(|(_, byte)| byte == &&b'\n')
            .enumerate()
            .last()
            .unwrap_or((0, (0, &0)));

        let line = last_newline.0;
        let col = byte_idx - last_newline.1 .0;

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
