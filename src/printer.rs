use crate::value::Value;

pub trait Printer {
    fn print(&self, value: Value);
}

pub struct ConsolePrinter;
impl Printer for ConsolePrinter {
    fn print(&self, value: Value) {
        println!("{}", value)
    }
}

#[cfg(test)]
pub mod vec_printer {
    use std::{cell::RefCell, rc::Rc};

    use crate::value::Value;

    use super::Printer;

    #[derive(Clone)]
    pub struct VecPrinter {
        lines: Rc<RefCell<Vec<Value>>>,
    }

    impl VecPrinter {
        pub fn new() -> Self {
            Self {
                lines: Rc::new(vec![].into()),
            }
        }
        pub fn get_lines(&self) -> Vec<Value> {
            self.lines.borrow().clone()
        }
    }

    impl Printer for VecPrinter {
        fn print(&self, value: Value) {
            self.lines.borrow_mut().push(value)
        }
    }
}
