pub trait ErrorReporter {
    fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message)
    }

    fn report(&mut self, line: usize, place: &str, message: &str);

    fn reset(&mut self);
    fn had_error(&self) -> bool;
}

pub struct ConsoleErrorReporter {
    had_error: bool,
}

impl ConsoleErrorReporter {
    pub fn new() -> Self {
        Self { had_error: false }
    }
}

impl ErrorReporter for ConsoleErrorReporter {
    fn report(&mut self, line: usize, place: &str, message: &str) {
        eprintln!("[line {}] Error {}: {}", line, place, message);
        self.had_error = true;
    }

    fn reset(&mut self) {
        self.had_error = false
    }
    fn had_error(&self) -> bool {
        self.had_error
    }
}

#[cfg(test)]
pub mod testing {
    use super::ErrorReporter;

    #[derive(Debug, PartialEq, Eq)]
    pub struct Logline {
        pub line: usize,
        pub place: String,
        pub message: String,
    }

    impl Logline {
        pub fn new(line: usize, place: &str, message: &str) -> Self {
            Self {
                line,
                place: place.into(),
                message: message.into(),
            }
        }
    }

    pub struct VectorErrorReporter {
        had_error: bool,
        errors: Vec<Logline>,
    }

    impl VectorErrorReporter {
        pub fn new() -> Self {
            Self {
                had_error: false,
                errors: Vec::new(),
            }
        }
        pub fn errors(&self) -> &Vec<Logline> {
            &self.errors
        }
    }

    impl ErrorReporter for VectorErrorReporter {
        fn report(&mut self, line: usize, place: &str, message: &str) {
            self.errors.push(Logline {
                line,
                place: place.into(),
                message: message.into(),
            });
            self.had_error = true;
        }

        fn reset(&mut self) {
            self.had_error = false
        }
        fn had_error(&self) -> bool {
            self.had_error
        }
    }
}
