use std::sync::Arc;

use miette::{Diagnostic, NamedSource, SourceSpan};

use crate::types::Type;

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum RuntimeError {
    #[error("Wrong operand type for operator {operator} : expected {expected} but got {actual}")]
    #[diagnostic(help("Change operand to {expected}"))]
    SingleTypeMissmatch {
        operator: String,
        expected: Type,
        actual: Type,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("operator")]
        operator_location: SourceSpan,
        #[label("operand has wrong type")]
        operand_location: SourceSpan,
    },
    #[error("Wrong operand types for operator {operator} : expected {expected} but got {actual_lhs} and {actual_rhs}")]
    #[diagnostic(help("Change operands to {expected}"))]
    DoubleTypeMissmatch {
        operator: String,
        expected: Type,
        actual_lhs: Type,
        actual_rhs: Type,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("operator")]
        operator_location: SourceSpan,
        #[label("operand has wrong type")]
        lhs: SourceSpan,
        #[label("operand has wrong type")]
        rhs: SourceSpan,
    },
}
