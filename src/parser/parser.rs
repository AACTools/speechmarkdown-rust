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

                    // Try to parse break: [time] or [break:...] or [break:"..."]
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

                    if found_bracket {
                        // Check if this is a break directive: [break:...]
                        if break_content.starts_with("break:") {
                            let break_value = break_content.trim_start_matches("break:");

                            // Remove quotes if present
                            let break_value = break_value.trim_matches('"').trim_matches('\'');

                            // Check if it's a time value or strength value
                            if Self::is_time_break(break_value) {
                                // Time-based break: [break:"3s"]
                                document = document.add_child(AstNode::new(NodeType::ShortBreak, format!("[{}]", break_value)));
                            } else {
                                // Strength-based break: [break:"strong"]
                                let mut node = AstNode::new(NodeType::Break, break_value.to_string());
                                node = node.with_attribute("strength", break_value);
                                document = document.add_child(node);
                            }
                        } else if Self::is_time_break(&break_content) {
                            // Simple time break: [3s]
                            document = document.add_child(AstNode::new(NodeType::ShortBreak, format!("[{}]", break_content)));
                        } else {
                            current_text.push('[');
                            current_text.push_str(&break_content);
                            current_text.push(']');
                        }
                    } else {
                        current_text.push('[');
                        current_text.push_str(&break_content);
                    }
                }
                '+' => {
                    // Check for emphasis: +text+ or ++text++
                    if !current_text.is_empty() {
                        document = document.add_child(AstNode::text(current_text.clone()));
                        current_text.clear();
                    }

                    // Count consecutive + signs to determine emphasis type
                    let mut plus_count = 1;
                    while chars.peek() == Some(&'+') {
                        chars.next();
                        plus_count += 1;
                    }

                    // Parse emphasized text
                    let mut emphasized_text = String::new();
                    let mut found_end = false;

                    while let Some(&next_c) = chars.peek() {
                        if next_c == '+' {
                            // Check if we have the right number of closing + signs
                            let mut closing_pluses = 0;
                            while chars.peek() == Some(&'+') {
                                chars.next();
                                closing_pluses += 1;
                            }

                            if closing_pluses == plus_count {
                                found_end = true;
                                break;
                            } else {
                                // Not the right number, add the + signs back as text
                                for _ in 0..closing_pluses {
                                    emphasized_text.push('+');
                                }
                            }
                        } else {
                            chars.next();
                            emphasized_text.push(next_c);
                        }
                    }

                    if found_end {
                        let node_type = match plus_count {
                            2 => NodeType::ShortEmphasisStrong,
                            1 => NodeType::ShortEmphasisModerate,
                            _ => NodeType::ShortEmphasisModerate,
                        };
                        document = document.add_child(AstNode::new(node_type, emphasized_text));
                    } else {
                        // Add the + signs and text as plain text
                        for _ in 0..plus_count {
                            current_text.push('+');
                        }
                        current_text.push_str(&emphasized_text);
                    }
                }
                '(' => {
                    // Check for text modifier: (text)[key:value] OR substitution: (text){alias}
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

                    if found_closing_paren {
                        // Check what comes next: [ for text modifier or { for substitution
                        if chars.peek() == Some(&'[') {
                            // Text modifier: (text)[modifiers]
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
                                current_text.push('[');
                                current_text.push_str(&modifiers);
                            }
                        } else if chars.peek() == Some(&'{') {
                            // Substitution: (text){alias}
                            chars.next(); // consume '{'

                            let mut alias_text = String::new();
                            let mut found_closing_brace = false;

                            while let Some(&next_c) = chars.peek() {
                                chars.next();
                                if next_c == '}' {
                                    found_closing_brace = true;
                                    break;
                                }
                                alias_text.push(next_c);
                            }

                            if found_closing_brace {
                                let mut node = AstNode::new(NodeType::ShortSub, modifier_content);
                                if !alias_text.is_empty() {
                                    node = node.with_attribute("alias", alias_text);
                                }
                                document = document.add_child(node);
                            } else {
                                current_text.push('(');
                                current_text.push_str(&modifier_content);
                                current_text.push(')');
                                current_text.push('{');
                                current_text.push_str(&alias_text);
                            }
                        } else {
                            // Just plain text with parentheses
                            current_text.push('(');
                            current_text.push_str(&modifier_content);
                            current_text.push(')');
                        }
                    } else {
                        current_text.push('(');
                        current_text.push_str(&modifier_content);
                    }
                }
                '/' => {
                    // Check for IPA notation: /phoneme/ or (/text/phoneme/)
                    if !current_text.is_empty() {
                        document = document.add_child(AstNode::text(current_text.clone()));
                        current_text.clear();
                    }

                    // Try to parse IPA: /phoneme/
                    let mut ipa_content = String::new();
                    let mut found_slash = false;
                    let mut slash_count = 1;

                    while let Some(&next_c) = chars.peek() {
                        chars.next();
                        if next_c == '/' {
                            slash_count += 1;
                            if slash_count == 2 {
                                found_slash = true;
                                break;
                            }
                        }
                        ipa_content.push(next_c);
                    }

                    if found_slash {
                        // Check if this is a bare IPA (/phoneme/) or part of a larger pattern
                        // For now, treat it as bare IPA
                        let mut node = AstNode::new(NodeType::BareIpa, "ipa".to_string());
                        node = node.with_attribute("alphabet", "ipa");
                        node = node.with_attribute("ph", ipa_content.trim().to_string());
                        document = document.add_child(node);
                    } else {
                        current_text.push('/');
                        current_text.push_str(&ipa_content);
                    }
                }
                '!' => {
                    // Check for audio: ![caption](url) or !(caption)["url"]
                    if chars.peek() == Some(&'[') {
                        // Handle ![caption](url) or ![url] format
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
                            // Handle ![caption](url) format
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
                        } else if found_caption_end {
                            // Handle ![url] format (url is in brackets, no caption)
                            // Check if the caption looks like a URL
                            let possible_url = caption.trim_matches('"').trim_matches('\'');
                            if possible_url.starts_with("http://") || possible_url.starts_with("https://") ||
                               possible_url.starts_with("soundbank://") || possible_url.contains("://") ||
                               possible_url.contains('.') {
                                let mut node = AstNode::new(NodeType::Audio, String::new());
                                node = node.with_attribute("src", possible_url);
                                document = document.add_child(node);
                            } else {
                                current_text.push_str(&format!("![{}]", caption));
                            }
                        } else {
                            current_text.push_str(&format!("![{}", caption));
                        }
                    } else if chars.peek() == Some(&'(') {
                        // Handle !(caption)["url"] format
                        if !current_text.is_empty() {
                            document = document.add_child(AstNode::text(current_text.clone()));
                            current_text.clear();
                        }

                        chars.next(); // consume '('
                        let mut caption = String::new();
                        let mut found_caption_end = false;

                        while let Some(&next_c) = chars.peek() {
                            chars.next();
                            if next_c == ')' {
                                found_caption_end = true;
                                break;
                            }
                            caption.push(next_c);
                        }

                        if found_caption_end && chars.peek() == Some(&'[') {
                            chars.next(); // consume '['
                            let mut url = String::new();
                            let mut found_url_end = false;

                            while let Some(&next_c) = chars.peek() {
                                chars.next();
                                if next_c == ']' {
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
                                current_text.push_str(&format!("(!{}[", caption));
                            }
                        } else {
                            current_text.push_str(&format!("!({}", caption));
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

    #[test]
    fn test_debug_substitution() {
        let input = "{Al}aluminum";
        let result = SpeechMarkdownParser::parse(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        println!("=== Substitution Debug ===");
        println!("Input: {}", input);
        println!("AST: {:?}", ast);
        println!("Children: {:?}", ast.children);
        println!("========================");
    }

    #[test]
    fn test_debug_emphasis_ssml() {
        let input = "++strong emphasis++";
        let result = SpeechMarkdownParser::to_ssml(input, crate::formatters::base::Platform::AmazonAlexa);
        println!("=== Emphasis SSML Debug ===");
        println!("Input: {}", input);
        println!("SSML Result: {:?}", result);
        println!("==========================");
    }
}