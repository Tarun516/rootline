//! Python language support: Tree-sitter parsing, module resolution, and
//! language facts. Parser and grammar types never leave this boundary;
//! downstream code depends on `rootline-core` IR contracts instead.
//!
//! The generic fact-graph builder in `crate::graph` orchestrates validated
//! construction; the Python-specific lookup policy it applies (scope rules,
//! receiver interpretation, builtin shadowing) will move behind an explicit
//! language-policy boundary once a second language arrives. Until then the
//! policy methods live with the builder, and the language knowledge
//! (grammar, builtins, module mapping) lives here.

mod builtins;
mod parse;
pub(crate) mod resolve;

pub(crate) use builtins::BUILTIN_NAMES;
pub use parse::{
    ADAPTER_NAME, ADAPTER_VERSION, PythonAdapter, PythonAdapterError, supports_extension,
};
pub use resolve::{ModuleIndex, Resolution};
