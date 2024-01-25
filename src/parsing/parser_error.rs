use std::sync::Arc;

use miette::{Diagnostic, NamedSource, SourceSpan};

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum ParserError {
    #[error("Expected )")]
    ExpectedRightParan {
        #[source_code]
        src: Arc<NamedSource>,
        #[label("here")]
        location: SourceSpan,
    },

    #[error("Expected expression")]
    ExpectedExpression {
        #[source_code]
        src: Arc<NamedSource>,
        #[label("here")]
        location: SourceSpan,
    },
}

#[derive(thiserror::Error, Debug, Diagnostic)]
#[error("Errors while parsing")]
pub struct ParserErrors {
    #[related]
    pub scanner_errors: Vec<ParserError>,
}
