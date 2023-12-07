#[derive(Default)]
pub struct ErrorReporter{
    had_error: bool,
}

impl ErrorReporter {
    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message)
    }

    pub fn report(&mut self, line: usize, place: &str, message: &str) {
        eprintln!("[line {}] Error {}: {}", line, place, message);
        self.had_error = true;
    }

    pub fn reset(&mut self) {
        self.had_error = false
    }
}