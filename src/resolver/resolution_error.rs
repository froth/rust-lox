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
    #[error("Can't return a value from initializer")]
    ReturnInInitializer {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("here")]
        location: SourceSpan,
    },
    #[error("Can't use 'this' outside of a class.")]
    InvalidThis {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("here")]
        location: SourceSpan,
    },
    #[error("A class can't inherit from itself")]
    SelfInheritance {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("here")]
        location: SourceSpan,
    },

    #[error("Can't use 'super' outside of a class.")]
    SuperOutsideClass {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("here")]
        location: SourceSpan,
    },

    #[error("Can't use 'super' in a class with no superclass")]
    SuperWithoutSuperclass {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("here")]
        location: SourceSpan,
    },
}
