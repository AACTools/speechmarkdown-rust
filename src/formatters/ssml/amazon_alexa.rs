use crate::ast::{AstNode, NodeType};
use crate::error::Result;
use crate::formatters::base::{Formatter, FormatterOptions};
use crate::formatters::ssml::base::SsmlFormatterBase;

pub struct AmazonAlexaSsmlFormatter {
    base: SsmlFormatterBase,
}

impl AmazonAlexaSsmlFormatter {
    pub fn new(options: FormatterOptions) -> Self {
        let base = SsmlFormatterBase::new(options);

        Self { base }
    }

    /// Override to add Amazon-specific functionality
    fn format_amazon_audio(&self, node: &AstNode) -> Result<String> {
        let src = node.attributes.get("src").unwrap_or(&String::new()).clone();

        let caption = &node.text;

        if caption.is_empty() {
            Ok(format!("<audio src=\"{}\"/>", src))
        } else {
            Ok(format!(
                "<audio src=\"{}\">\n<desc>{}</desc>\n</audio>",
                src,
                self.base.escape_xml(caption)
            ))
        }
    }

    /// Format Amazon-specific emotion tags
    fn format_amazon_emotion(&self, node: &AstNode) -> Result<String> {
        let emotion_type = match node.node_type {
            NodeType::Excited => "excited",
            NodeType::Disappointed => "disappointed",
            _ => return Ok(String::new()),
        };

        let intensity = node
            .attributes
            .get("value")
            .unwrap_or(&"medium".to_string())
            .clone();

        Ok(format!(
            "<amazon:emotion name=\"{}\" intensity=\"{}\">",
            emotion_type, intensity
        ))
    }

    /// Format Amazon-specific domain tags
    fn format_amazon_domain(&self, node: &AstNode) -> Result<String> {
        let domain_name = match node.node_type {
            NodeType::Newscaster => "news",
            NodeType::Dj => "music",
            _ => return Ok(String::new()),
        };

        Ok(format!("<amazon:domain name=\"{}\">", domain_name))
    }
}

impl Formatter for AmazonAlexaSsmlFormatter {
    fn format(&self, ast: &AstNode) -> Result<String> {
        self.base.format(ast)
    }

    fn format_node(&self, node: &AstNode) -> Result<String> {
        match node.node_type {
            // Audio elements
            NodeType::Audio => self.format_amazon_audio(node),

            // Amazon-specific emotions
            NodeType::Excited | NodeType::Disappointed => self.format_amazon_emotion(node),

            // Amazon-specific domains
            NodeType::Newscaster | NodeType::Dj => self.format_amazon_domain(node),

            // Use base formatter for everything else
            _ => self.base.format_node(node),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::SpeechMarkdownParser;

    #[test]
    fn test_amazon_alexa_basic_parsing() {
        let input = "Hello world";
        let result =
            SpeechMarkdownParser::to_ssml(input, crate::formatters::base::Platform::AmazonAlexa);
        assert!(result.is_ok());
    }

    #[test]
    fn test_amazon_alexa_with_break() {
        let input = "Sample [2s] text";
        let result =
            SpeechMarkdownParser::to_ssml(input, crate::formatters::base::Platform::AmazonAlexa);
        assert!(result.is_ok());

        let ssml = result.unwrap();
        assert!(ssml.contains("<break time=\"2s\"/>"));
    }

    #[test]
    fn test_amazon_alexa_with_emphasis() {
        let input = "++strong emphasis++";
        let result =
            SpeechMarkdownParser::to_ssml(input, crate::formatters::base::Platform::AmazonAlexa);
        assert!(result.is_ok());

        let ssml = result.unwrap();
        assert!(ssml.contains("<emphasis level=\"strong\">"));
    }
}
