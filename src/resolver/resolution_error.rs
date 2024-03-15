use std::sync::Arc;

use miette::{Diagnostic, NamedSource, SourceSpan};

use crate::ast::name::Name;

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum ResolutionError {
    #[error("Can't read local variable \"{name}\" in its own initializer")]
    InitializedWithSelf {
        name: Name,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("here")]
        location: SourceSpan,
    },
    #[error("Can't return from top-level code")]
    InvalidReturn {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("here")]
        location: SourceSpan,
    },
}
