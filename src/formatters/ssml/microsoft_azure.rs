use crate::ast::{AstNode, NodeType};
use crate::error::Result;
use crate::formatters::base::{Formatter, FormatterOptions};
use crate::formatters::ssml::base::SsmlFormatterBase;

pub struct MicrosoftAzureSsmlFormatter {
    base: SsmlFormatterBase,
    options: FormatterOptions,
}

impl MicrosoftAzureSsmlFormatter {
    pub fn new(options: FormatterOptions) -> Self {
        let base = SsmlFormatterBase::new(options.clone());

        Self { base, options }
    }

    fn format_azure_section_open(&self, node: &AstNode) -> Result<String> {
        let style = node
            .attributes
            .get("style")
            .or_else(|| node.attributes.get("emotion"))
            .unwrap_or(&node.text)
            .clone();

        Ok(format!("<mstts:express-as style=\"{}\">", style))
    }

    fn format_azure_section_close(&self) -> String {
        "</mstts:express-as>".to_string()
    }

    fn format_document_azure(&self, node: &AstNode) -> Result<String> {
        let mut content = String::new();
        let mut chars = node.children.iter().peekable();
        let mut in_section = false;

        while let Some(child) = chars.next() {
            if child.node_type == NodeType::Section {
                if in_section {
                    content.push_str(&self.format_azure_section_close());
                    content.push('\n');
                }

                content.push('\n');
                content.push_str(&self.format_azure_section_open(child)?);
                in_section = true;

                while let Some(next_child) = chars.peek() {
                    if next_child.node_type == NodeType::Section {
                        break;
                    }
                    let next_child = chars.next().unwrap();
                    content.push_str(&self.base.format_node(next_child)?);
                }
            } else {
                content.push_str(&self.base.format_node(child)?);
            }
        }

        if in_section {
            content.push('\n');
            content.push_str(&self.format_azure_section_close());
            content.push('\n');
        }

        if self.options.include_speak_tag {
            Ok(format!(
                "<speak xmlns:mstts=\"https://www.w3.org/2001/mstts\">\n{}\n</speak>",
                content
            ))
        } else {
            Ok(content)
        }
    }
}

impl Formatter for MicrosoftAzureSsmlFormatter {
    fn format(&self, ast: &AstNode) -> Result<String> {
        self.format_document_azure(ast)
    }

    fn format_node(&self, node: &AstNode) -> Result<String> {
        match node.node_type {
            NodeType::Section => self.format_azure_section_open(node),
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
        let result =
            SpeechMarkdownParser::to_ssml(input, crate::formatters::base::Platform::MicrosoftAzure);
        assert!(result.is_ok());
    }

    #[test]
    fn test_microsoft_azure_with_section() {
        let input = "#[angry] I am angry!";
        let result =
            SpeechMarkdownParser::to_ssml(input, crate::formatters::base::Platform::MicrosoftAzure);
        assert!(result.is_ok());

        let ssml = result.unwrap();
        assert!(ssml.contains("<mstts:express-as"));
        assert!(ssml.contains("style=\"angry\""));
        assert!(ssml.contains("xmlns:mstts"));
    }
}
