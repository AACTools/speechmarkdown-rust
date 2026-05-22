use crate::ast::AstNode;
use crate::error::Result;
use std::collections::HashMap;

/// Supported TTS platforms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    AmazonAlexa,
    GoogleAssistant,
    MicrosoftAzure,
    Apple,
    W3c,
    SamsungBixby,
    ElevenLabs,
    IbmWatson,
}

impl Platform {
    /// Parse platform from string
    pub fn from_platform_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "amazon-alexa" | "alexa" => Some(Platform::AmazonAlexa),
            "google-assistant" | "google" => Some(Platform::GoogleAssistant),
            "microsoft-azure" | "azure" => Some(Platform::MicrosoftAzure),
            "apple" => Some(Platform::Apple),
            "w3c" => Some(Platform::W3c),
            "samsung-bixby" | "bixby" => Some(Platform::SamsungBixby),
            "elevenlabs" => Some(Platform::ElevenLabs),
            "ibm-watson" | "watson" => Some(Platform::IbmWatson),
            _ => None,
        }
    }

    /// Convert platform to string identifier
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::AmazonAlexa => "amazon-alexa",
            Platform::GoogleAssistant => "google-assistant",
            Platform::MicrosoftAzure => "microsoft-azure",
            Platform::Apple => "apple",
            Platform::W3c => "w3c",
            Platform::SamsungBixby => "samsung-bixby",
            Platform::ElevenLabs => "elevenlabs",
            Platform::IbmWatson => "ibm-watson",
        }
    }
}

/// Voice mapping for custom voice names
#[derive(Debug, Clone)]
pub struct VoiceMapping {
    pub voice_attributes: HashMap<String, String>,
}

/// Options for formatters
#[derive(Debug, Clone)]
pub struct FormatterOptions {
    pub platform: Platform,
    pub include_speak_tag: bool,
    pub include_paragraph_tag: bool,
    pub preserve_empty_lines: bool,
    pub escape_xml_symbols: bool,
    pub include_formatter_comment: bool,
    pub voices: Option<HashMap<String, VoiceMapping>>,
}

impl Default for FormatterOptions {
    fn default() -> Self {
        Self {
            platform: Platform::AmazonAlexa,
            include_speak_tag: true,
            include_paragraph_tag: false,
            preserve_empty_lines: true,
            escape_xml_symbols: false,
            include_formatter_comment: false,
            voices: None,
        }
    }
}

/// Base trait for all formatters
pub trait Formatter {
    /// Format an AST into a string
    fn format(&self, ast: &AstNode) -> Result<String>;

    /// Format a single node
    fn format_node(&self, node: &AstNode) -> Result<String>;
}

/// Create a formatter for the specified platform
pub fn create_formatter(platform: Platform, options: FormatterOptions) -> Box<dyn Formatter> {
    match platform {
        Platform::AmazonAlexa => Box::new(super::ssml::AmazonAlexaSsmlFormatter::new(options)),
        Platform::GoogleAssistant => {
            Box::new(super::ssml::GoogleAssistantSsmlFormatter::new(options))
        }
        Platform::MicrosoftAzure => {
            Box::new(super::ssml::MicrosoftAzureSsmlFormatter::new(options))
        }
        Platform::Apple => {
            // For now, use the base SSML formatter
            Box::new(super::ssml::SsmlFormatterBase::new(options))
        }
        Platform::W3c => {
            // Use the base SSML formatter for W3C standard
            Box::new(super::ssml::SsmlFormatterBase::new(options))
        }
        _ => Box::new(super::TextFormatter::new()),
    }
}
