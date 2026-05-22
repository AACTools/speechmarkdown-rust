use thiserror::Error;

/// Result type for SpeechMarkdown operations
pub type Result<T> = std::result::Result<T, ParseError>;

/// Errors that can occur during SpeechMarkdown parsing and formatting
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Grammar parsing error
    #[error("Grammar error: {0}")]
    GrammarError(String),

    /// Invalid modifier
    #[error("Invalid modifier: {0}")]
    InvalidModifier(String),

    /// Invalid modifier value
    #[error("Invalid value for modifier '{modifier}': {value}")]
    InvalidModifierValue { modifier: String, value: String },

    /// Unsupported platform
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    /// Invalid voice name for platform
    #[error("Invalid voice '{voice}' for platform '{platform}'")]
    InvalidVoice { voice: String, platform: String },

    /// Invalid language code
    #[error("Invalid language code: {0}")]
    InvalidLanguage(String),

    /// Invalid intensity value
    #[error("Invalid intensity: {0}")]
    InvalidIntensity(String),

    /// IO error for file operations
    #[error("IO error: {0}")]
    IoError(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::JsonError(err.to_string())
    }
}
