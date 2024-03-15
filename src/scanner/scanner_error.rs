use std::{num::ParseFloatError, sync::Arc};

use miette::{Diagnostic, NamedSource, SourceSpan};

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum ScannerError {
    #[error("Unexpected character: {char}")]
    UnexpectedCharacter {
        char: char,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("here")]
        location: SourceSpan,
    },

    #[error("Unexpected characters: {chars}")]
    UnexpectedCharacters {
        chars: String,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("here")]
        location: SourceSpan,
    },

    #[error("Non terminated String")]
    NonTerminatedString {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("here")]
        location: SourceSpan,
    },

    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
}

#[derive(thiserror::Error, Debug, Diagnostic)]
#[error("Errors while scanning")]
pub struct ScannerErrors {
    #[related]
    pub scanner_errors: Vec<ScannerError>,
}
