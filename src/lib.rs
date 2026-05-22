// SpeechMarkdown Parser - High-performance SpeechMarkdown parser with multi-language bindings

pub mod ast;
pub mod error;
pub mod ffi;
pub mod formatters;
pub mod parser;

// Re-export main types for convenience
pub use ast::{AstNode, NodeType, Position};
pub use error::{ParseError, Result};
pub use formatters::base::{create_formatter, Formatter, FormatterOptions, Platform};
pub use parser::SpeechMarkdownParser;
