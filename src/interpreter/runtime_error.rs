use std::sync::Arc;

use miette::{Diagnostic, NamedSource, SourceSpan};

use crate::{ast::expr::Name, types::Type};

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
}
