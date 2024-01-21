use std::num::ParseFloatError;

use miette::{Diagnostic, NamedSource, SourceSpan};

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Scanner(#[from] ScannerError),
}

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum ScannerError {
    /// For starter, to remove as code matures.
    #[error("Generic error: {0}")]
    Generic(String),
    
    #[error("Non terminated String")]
    NonTerminatedString{
        // The Source that we're gonna be printing snippets out of.
        // This can be a String if you don't have or care about file names.
        #[source_code]
        src: NamedSource,
        // Snippets and highlights can be included in the diagnostic!
        #[label("here")]
        location: SourceSpan,
    },

    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
}
