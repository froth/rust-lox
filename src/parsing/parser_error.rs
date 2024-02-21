use std::sync::Arc;

use miette::{Diagnostic, NamedSource, SourceSpan};

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum ParserError {
    #[error("Expected )")]
    ExpectedRightParan {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("here")]
        location: SourceSpan,
    },

    #[error("Expected expression")]
    ExpectedExpression {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("here")]
        location: SourceSpan,
    },

    #[error("Unexpected EOF")]
    UnexpectedEof {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("here")]
        location: SourceSpan,
    },

    #[error("Expected ';' after expression")]
    ExpectedSemicolon {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("expression")]
        location: SourceSpan,
    },
}

#[derive(thiserror::Error, Debug, Diagnostic)]
#[error("Errors while parsing")]
pub struct ParserErrors {
    #[related]
    pub scanner_errors: Vec<ParserError>,
}
