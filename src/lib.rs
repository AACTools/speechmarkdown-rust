// SpeechMarkdown Parser - High-performance SpeechMarkdown parser with multi-language bindings

pub mod ast;
pub mod capabilities;
pub mod error;
pub mod ffi;
pub mod formatters;
pub mod parser;
pub mod ssml_to_smd;

// Re-export main types for convenience
pub use ast::{AstNode, NodeType, Position};
pub use capabilities::{get_supported_ssml, PlatformCapabilities, SsmlCapability};
pub use error::{ParseError, Result};
pub use formatters::base::{create_formatter, Formatter, FormatterOptions, Platform};
pub use parser::SpeechMarkdownParser;
