use std::sync::Arc;

use miette::{Diagnostic, NamedSource, SourceSpan};

use crate::types::Type;

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum RuntimeError {
    #[error("Wrong operand type of unary {operator} : expected {expected} but got {actual}")]
    #[diagnostic(help("Change operand to {expected}"))]
    UnaryTypeMissmatch {
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
}
