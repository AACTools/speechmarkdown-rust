use crate::ast::{AstNode, NodeType};
use crate::error::Result;
use crate::formatters::base::Formatter;

/// Plain text formatter - strips all markup
pub struct TextFormatter {
    preserve_empty_lines: bool,
}

impl TextFormatter {
    pub fn new() -> Self {
        Self {
            preserve_empty_lines: true,
        }
    }

    pub fn with_options(preserve_empty_lines: bool) -> Self {
        Self {
            preserve_empty_lines,
        }
    }
}

impl Default for TextFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Formatter for TextFormatter {
    fn format(&self, ast: &AstNode) -> Result<String> {
        let mut result = Vec::new();
        self.format_node_recursive(ast, &mut result);
        let text = result.join("");

        // Clean up whitespace
        let text = self.clean_whitespace(&text);

        Ok(text)
    }

    fn format_node(&self, node: &AstNode) -> Result<String> {
        let mut result = Vec::new();
        self.format_node_recursive(node, &mut result);
        Ok(result.join(""))
    }
}

impl TextFormatter {
    fn format_node_recursive(&self, node: &AstNode, result: &mut Vec<String>) {
        match node.node_type {
            // Structural nodes - process children
            NodeType::Document | NodeType::Paragraph | NodeType::SimpleLine => {
                for child in &node.children {
                    self.format_node_recursive(child, result);
                }
            }

            // Empty lines
            NodeType::EmptyLine => {
                if self.preserve_empty_lines {
                    result.push("\n\n".to_string());
                } else {
                    result.push("\n".to_string());
                }
            }

            // Plain text content
            NodeType::PlainText | NodeType::PlainTextSpecialChars | NodeType::PlainTextEmphasis => {
                result.push(node.text.clone());
            }

            // Breaks - add space
            NodeType::ShortBreak | NodeType::Break => {
                result.push(" ".to_string());
            }

            // Emphasis - just use the text
            NodeType::ShortEmphasisModerate
            | NodeType::ShortEmphasisStrong
            | NodeType::ShortEmphasisNone
            | NodeType::ShortEmphasisReduced => {
                result.push(node.text.clone());
            }

            // Text modifiers - extract the text content
            NodeType::TextModifier => {
                // For text modifiers, the text is stored in the node's text field
                result.push(node.text.clone());
            }

            // IPA - use the text content (pronunciation)
            NodeType::ShortIpa => {
                result.push(node.text.clone());
            }
            NodeType::BareIpa => {
                if let Some(ph) = node.attributes.get("ph") {
                    result.push(ph.clone());
                } else {
                    result.push(node.text.clone());
                }
            }

            // Substitution - use the alias if available, otherwise the text
            NodeType::ShortSub => {
                result.push(node.text.clone());
            }

            // Audio - no output in plain text
            NodeType::Audio => {
                // Audio elements produce no text output
            }

            // Mark tags - no output
            NodeType::Mark => {
                // Mark tags produce no output
            }

            // Modifiers are handled as part of text modifiers
            NodeType::Emphasis
            | NodeType::Voice
            | NodeType::Lang
            | NodeType::Rate
            | NodeType::Pitch
            | NodeType::Volume
            | NodeType::Whisper
            | NodeType::Excited
            | NodeType::Disappointed
            | NodeType::Newscaster
            | NodeType::Dj
            | NodeType::Date
            | NodeType::Time
            | NodeType::Number
            | NodeType::Ordinal
            | NodeType::Characters
            | NodeType::Fraction
            | NodeType::Telephone
            | NodeType::Unit
            | NodeType::Address
            | NodeType::Interjection
            | NodeType::Expletive
            | NodeType::Ipa
            | NodeType::Sub => {
                // These are handled as part of text modifiers, not standalone
            }

            // Section - process children
            NodeType::Section => {
                for child in &node.children {
                    self.format_node_recursive(child, result);
                }
            }
        }
    }

    fn clean_whitespace(&self, text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let cleaned: Vec<String> = lines
            .iter()
            .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|line| !line.is_empty())
            .collect();
        let result = cleaned.join("\n");
        result.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::SpeechMarkdownParser;

    #[test]
    fn test_format_plain_text() {
        let ast = SpeechMarkdownParser::parse("Hello world").unwrap();

        let formatter = TextFormatter::new();
        let result = formatter.format(&ast).unwrap();

        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_format_with_breaks() {
        let ast = SpeechMarkdownParser::parse("Sample [2s] text").unwrap();

        let formatter = TextFormatter::new();
        let result = formatter.format(&ast).unwrap();

        assert_eq!(result, "Sample text");
    }

    #[test]
    fn test_format_with_emphasis() {
        let ast = SpeechMarkdownParser::parse("++strong emphasis++").unwrap();

        let formatter = TextFormatter::new();
        let result = formatter.format(&ast).unwrap();

        assert_eq!(result, "strong emphasis");
    }

    #[test]
    fn test_format_with_text_modifier() {
        let ast = SpeechMarkdownParser::parse("(text)[voice:\"Kendra\"]").unwrap();

        let formatter = TextFormatter::new();
        let result = formatter.format(&ast).unwrap();

        assert_eq!(result, "text");
    }

    #[test]
    fn test_format_with_substitution() {
        let input = "{Al}aluminum";
        let ast = SpeechMarkdownParser::parse(input).unwrap();

        let formatter = TextFormatter::new();
        let result = formatter.format(&ast).unwrap();

        assert_eq!(result, "Al");
    }

    #[test]
    fn test_format_complex_sentence() {
        let ast = SpeechMarkdownParser::parse("Why do you keep switching voices (from one)[voice:\"Brian\"] to (the other)[voice:\"Kendra\"]?").unwrap();

        let formatter = TextFormatter::new();
        let result = formatter.format(&ast).unwrap();

        assert_eq!(
            result,
            "Why do you keep switching voices from one to the other?"
        );
    }

    #[test]
    fn test_format_with_audio() {
        let ast =
            SpeechMarkdownParser::parse("Hello ![sound](\"https://example.com/audio.mp3\") world")
                .unwrap();

        let formatter = TextFormatter::new();
        let result = formatter.format(&ast).unwrap();

        assert_eq!(result, "Hello world");
    }
}
