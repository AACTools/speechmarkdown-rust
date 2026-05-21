use crate::ast::{AstNode, NodeType};
use crate::error::Result;
use crate::formatters::{TextFormatter, create_formatter, Formatter};
use crate::formatters::base::{FormatterOptions, Platform};

pub struct SpeechMarkdownParser;

impl SpeechMarkdownParser {
    /// Parse SpeechMarkdown text into an AST
    pub fn parse(input: &str) -> Result<AstNode> {
        Self::parse_simple(input)
    }

    /// Convert SpeechMarkdown to plain text
    pub fn to_text(input: &str) -> Result<String> {
        let ast = Self::parse(input)?;
        let formatter = TextFormatter::new();
        formatter.format(&ast)
    }

    /// Convert SpeechMarkdown to SSML for the specified platform
    pub fn to_ssml(input: &str, platform: Platform) -> Result<String> {
        let ast = Self::parse(input)?;
        let options = FormatterOptions {
            platform,
            ..Default::default()
        };
        let formatter = create_formatter(platform, options);
        formatter.format(&ast)
    }

    /// Simple manual parser for basic SpeechMarkdown syntax
    fn parse_simple(input: &str) -> Result<AstNode> {
        let mut document = AstNode::document();
        let mut current_text = String::new();
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '[' => {
                    // Check for break notation
                    if !current_text.is_empty() {
                        document = document.add_child(AstNode::text(current_text.clone()));
                        current_text.clear();
                    }

                    // Try to parse break: [time]
                    let mut break_content = String::new();
                    let mut found_bracket = false;

                    while let Some(&next_c) = chars.peek() {
                        chars.next();
                        if next_c == ']' {
                            found_bracket = true;
                            break;
                        }
                        break_content.push(next_c);
                    }

                    if found_bracket && Self::is_time_break(&break_content) {
                        document = document.add_child(AstNode::new(NodeType::ShortBreak, format!("[{}]", break_content)));
                    } else {
                        current_text.push('[');
                        current_text.push_str(&break_content);
                        if found_bracket {
                            current_text.push(']');
                        }
                    }
                }
                '+' => {
                    // Check for emphasis: +text+ or ++text++
                    if !current_text.is_empty() {
                        document = document.add_child(AstNode::text(current_text.clone()));
                        current_text.clear();
                    }

                    let prev_was_plus = current_text.ends_with('+');
                    let emphasis_count = if prev_was_plus { 2 } else { 1 };

                    // Parse emphasized text
                    let mut emphasized_text = String::new();
                    let mut found_end = false;

                    while let Some(&next_c) = chars.peek() {
                        chars.next();
                        if next_c == '+' {
                            if emphasis_count == 2 && chars.peek() == Some(&'+') {
                                chars.next(); // consume second +
                                found_end = true;
                                break;
                            } else if emphasis_count == 1 {
                                found_end = true;
                                break;
                            }
                        }
                        emphasized_text.push(next_c);
                    }

                    if found_end {
                        let node_type = if emphasis_count == 2 {
                            NodeType::ShortEmphasisStrong
                        } else {
                            NodeType::ShortEmphasisModerate
                        };
                        document = document.add_child(AstNode::new(node_type, emphasized_text));
                    } else {
                        current_text.push('+');
                        current_text.push_str(&emphasized_text);
                    }
                }
                '(' => {
                    // Check for text modifier: (text)[key:value]
                    if !current_text.is_empty() {
                        document = document.add_child(AstNode::text(current_text.clone()));
                        current_text.clear();
                    }

                    let mut modifier_content = String::new();
                    let mut found_closing_paren = false;

                    while let Some(&next_c) = chars.peek() {
                        chars.next();
                        if next_c == ')' {
                            found_closing_paren = true;
                            break;
                        }
                        modifier_content.push(next_c);
                    }

                    if found_closing_paren && chars.peek() == Some(&'[') {
                        chars.next(); // consume '['

                        let mut modifiers = String::new();
                        let mut found_closing_bracket = false;

                        while let Some(&next_c) = chars.peek() {
                            chars.next();
                            if next_c == ']' {
                                found_closing_bracket = true;
                                break;
                            }
                            modifiers.push(next_c);
                        }

                        if found_closing_bracket {
                            let mut node = AstNode::new(NodeType::TextModifier, modifier_content);
                            // Parse modifiers
                            for modifier in modifiers.split(';') {
                                if let Some((key, value)) = modifier.split_once(':') {
                                    node = node.with_attribute(key.trim(), value.trim().trim_matches('"').trim_matches('\''));
                                } else {
                                    node = node.with_attribute(modifier.trim(), "");
                                }
                            }
                            document = document.add_child(node);
                        } else {
                            current_text.push('(');
                            current_text.push_str(&modifier_content);
                            current_text.push(')');
                            current_text.push('[');
                            current_text.push_str(&modifiers);
                        }
                    } else {
                        current_text.push('(');
                        current_text.push_str(&modifier_content);
                    }
                }
                '!' => {
                    // Check for audio: ![caption](url)
                    if chars.peek() == Some(&'[') {
                        if !current_text.is_empty() {
                            document = document.add_child(AstNode::text(current_text.clone()));
                            current_text.clear();
                        }

                        chars.next(); // consume '['
                        let mut caption = String::new();
                        let mut found_caption_end = false;

                        while let Some(&next_c) = chars.peek() {
                            chars.next();
                            if next_c == ']' {
                                found_caption_end = true;
                                break;
                            }
                            caption.push(next_c);
                        }

                        if found_caption_end && chars.peek() == Some(&'(') {
                            chars.next(); // consume '('
                            let mut url = String::new();
                            let mut found_url_end = false;

                            while let Some(&next_c) = chars.peek() {
                                chars.next();
                                if next_c == ')' {
                                    found_url_end = true;
                                    break;
                                }
                                url.push(next_c);
                            }

                            if found_url_end {
                                let mut node = AstNode::new(NodeType::Audio, caption);
                                node = node.with_attribute("src", url.trim_matches('"').trim_matches('\''));
                                document = document.add_child(node);
                            } else {
                                current_text.push_str(&format!("![{}]", caption));
                            }
                        } else {
                            current_text.push_str(&format!("![{}", caption));
                        }
                    } else {
                        current_text.push('!');
                    }
                }
                ' ' | '\t' | '\n' | '\r' => {
                    current_text.push(c);
                }
                _ => {
                    current_text.push(c);
                }
            }
        }

        // Add remaining text
        if !current_text.is_empty() {
            document = document.add_child(AstNode::text(current_text));
        }

        Ok(document)
    }

    fn is_time_break(s: &str) -> bool {
        s.ends_with("s") || s.ends_with("ms")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_text() {
        let result = SpeechMarkdownParser::parse("Hello world");
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.node_type, NodeType::Document);
        assert!(!ast.children.is_empty());
    }

    #[test]
    fn test_parse_short_break() {
        let result = SpeechMarkdownParser::parse("Sample [2s] text");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_emphasis_strong() {
        let result = SpeechMarkdownParser::parse("++strong emphasis++");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_text_modifier() {
        let result = SpeechMarkdownParser::parse("(text)[voice:\"Kendra\"]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_audio() {
        let result = SpeechMarkdownParser::parse("![caption](\"https://example.com/audio.mp3\")");
        assert!(result.is_ok());
    }
}