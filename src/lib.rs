// SpeechMarkdown Parser - High-performance SpeechMarkdown parser with multi-language bindings

pub mod ast;
pub mod error;
pub mod parser;
pub mod formatters;

// Re-export main types for convenience
pub use ast::{AstNode, NodeType, Position};
pub use error::{ParseError, Result};
pub use parser::SpeechMarkdownParser;
pub use formatters::base::{Platform, Formatter, FormatterOptions, create_formatter};
