use crate::ast::{AstNode, NodeType};
use crate::error::Result;
use crate::formatters::ssml::base::SsmlFormatterBase;
use crate::formatters::base::{Formatter, FormatterOptions};

pub struct MicrosoftAzureSsmlFormatter {
    base: SsmlFormatterBase,
}

impl MicrosoftAzureSsmlFormatter {
    pub fn new(options: FormatterOptions) -> Self {
        let base = SsmlFormatterBase::new(options);

        Self { base }
    }

    /// Format Azure-specific section markers (style, emotion, etc.)
    fn format_azure_section(&self, node: &AstNode) -> Result<String> {
        let style = node.attributes.get("style")
            .or_else(|| node.attributes.get("emotion"))
            .unwrap_or(&node.text)
            .clone();

        Ok(format!("<mstts:express-as style=\"{}\">", style))
    }
}

impl Formatter for MicrosoftAzureSsmlFormatter {
    fn format(&self, ast: &AstNode) -> Result<String> {
        // Add Azure-specific namespace to speak tag
        let ssml = self.base.format(ast)?;

        // Check if we need to add the Azure namespace
        if ssml.contains("<speak>") && !ssml.contains("xmlns:mstts") {
            let ssml = ssml.replace("<speak>", "<speak xmlns:mstts=\"https://www.w3.org/2001/mstts\">");
            Ok(ssml)
        } else {
            Ok(ssml)
        }
    }

    fn format_node(&self, node: &AstNode) -> Result<String> {
        match node.node_type {
            // Azure-specific sections
            NodeType::Section => self.format_azure_section(node),

            // Use base formatter for everything else
            _ => self.base.format_node(node),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::SpeechMarkdownParser;

    #[test]
    fn test_microsoft_azure_basic_parsing() {
        let input = "Hello world";
        let result = SpeechMarkdownParser::to_ssml(input, crate::formatters::base::Platform::MicrosoftAzure);
        assert!(result.is_ok());
    }

    #[test]
    fn test_microsoft_azure_with_section() {
        let input = "#[angry] I am angry!";
        let result = SpeechMarkdownParser::to_ssml(input, crate::formatters::base::Platform::MicrosoftAzure);
        assert!(result.is_ok());

        let ssml = result.unwrap();
        assert!(ssml.contains("<mstts:express-as"));
        assert!(ssml.contains("style=\"angry\""));
        assert!(ssml.contains("xmlns:mstts"));
    }
}