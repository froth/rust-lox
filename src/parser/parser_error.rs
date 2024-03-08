use std::sync::Arc;

use miette::{Diagnostic, NamedSource, SourceSpan};

use crate::ast::expr::Expr;

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum ParserError {
    #[error("Expected (")]
    ExpectedLeftParen {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("here")]
        location: SourceSpan,
    },
    #[error("Expected )")]
    ExpectedRightParen {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("here")]
        location: SourceSpan,
    },

    #[error("Expected }}")]
    ExpectedRightBrace {
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
        expr: Option<Expr>, //for interpreting expr in repl without
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("; expected")]
        location: SourceSpan,
    },

    #[error("Expected identifier")]
    ExpectedIdentifier {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("no identifier")]
        location: SourceSpan,
    },

    #[error("Invalid assignment target")]
    InvalidAssignmentTarget {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("not a valid target")]
        location: SourceSpan,
    },

    #[error("Too many arguments (max 255)")]
    TooManyArguments {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("this one is one too many")]
        location: SourceSpan,
    },
}

#[derive(thiserror::Error, Debug, Diagnostic)]
#[error("Errors while parsing")]
pub struct ParserErrors {
    #[related]
    pub parser_errors: Vec<ParserError>,
}
