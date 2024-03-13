use std::sync::Arc;

use miette::{Diagnostic, NamedSource, SourceSpan};

use crate::{ast::name::Name, interpreter::types::Type};

use super::value::Value;

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum RuntimeError {
    #[error("Wrong operand type for operator {operator} : expected {expected} but got {actual}")]
    #[diagnostic(help("Change operand to {expected}"))]
    WrongType {
        operator: String,
        expected: Type,
        actual: Type,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("operator")]
        operator_location: SourceSpan,
        #[label("{actual}")]
        operand_location: SourceSpan,
    },
    #[error("Wrong operand types for operator {operator} : expected {expected} but got {actual_lhs} and {actual_rhs}")]
    #[diagnostic(help("Change operands to {expected}"))]
    WrongTypes {
        operator: String,
        expected: Type,
        actual_lhs: Type,
        actual_rhs: Type,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("operator")]
        operator_location: SourceSpan,
        #[label("{actual_lhs}")]
        lhs: SourceSpan,
        #[label("{actual_rhs}")]
        rhs: SourceSpan,
    },
    #[error("Wrong operand types for operator + : expected both String of both Number but got {actual_lhs} and {actual_rhs}")]
    #[diagnostic(help("Change operands to be both String or Number"))]
    PlusOperatorWrongTypes {
        actual_lhs: Type,
        actual_rhs: Type,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("operator")]
        operator_location: SourceSpan,
        #[label("{actual_lhs}")]
        lhs: SourceSpan,
        #[label("{actual_rhs}")]
        rhs: SourceSpan,
    },

    #[error("Undefined variable '{name}'")]
    UndefinedVariable {
        name: Name,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("here")]
        location: SourceSpan,
    },

    #[error("Can only call functions and classes but got {actual}")]
    CallingNonCallable {
        actual: Type,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("not callable {actual}, change to class of function")]
        location: SourceSpan,
    },

    #[error("Expected {expected} arguments but got {actual}")]
    WrongArity {
        expected: usize,
        actual: usize,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("here")]
        location: SourceSpan,
    },
}

#[derive(Debug)]
pub(super) enum RuntimeErrorOrReturn {
    RuntimeError(RuntimeError),
    Return(Value),
}

impl From<RuntimeError> for RuntimeErrorOrReturn {
    fn from(value: RuntimeError) -> Self {
        RuntimeErrorOrReturn::RuntimeError(value)
    }
}

impl RuntimeErrorOrReturn {
    pub(super) fn unwrap_runtime_error(self) -> RuntimeError {
        match self {
            RuntimeErrorOrReturn::RuntimeError(runtime_error) => runtime_error,
            RuntimeErrorOrReturn::Return(_) => {
                panic!("Return can only be in functions: guaranteed by static analysis")
            }
        }
    }
}
