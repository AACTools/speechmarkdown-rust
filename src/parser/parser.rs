use crate::ast::{AstNode, NodeType};
use crate::error::Result;
use crate::formatters::base::{FormatterOptions, Platform};
use crate::formatters::{create_formatter, Formatter, TextFormatter};
use crate::ssml_to_smd;

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

    /// Convert SSML to SpeechMarkdown (best-effort, lossy for unsupported elements)
    pub fn to_smd(ssml: &str) -> Result<String> {
        ssml_to_smd::ssml_to_smd(ssml)
    }

    /// Check if a string contains SpeechMarkdown syntax
    pub fn is_speech_markdown(input: &str) -> bool {
        if let Ok(ast) = Self::parse(input) {
            ast.children.iter().any(|child| {
                !matches!(
                    child.node_type,
                    NodeType::Document | NodeType::PlainText | NodeType::EmptyLine
                )
            })
        } else {
            false
        }
    }

    /// Validate SpeechMarkdown input, returning an error message if invalid
    pub fn validate(input: &str) -> Result<()> {
        Self::parse(input)?;
        Ok(())
    }

    /// Simple manual parser for basic SpeechMarkdown syntax
    fn parse_simple(input: &str) -> Result<AstNode> {
        let mut document = AstNode::document();
        let mut current_text = String::new();
        let mut chars = input.chars().peekable();

        let flush_text = |doc: &mut AstNode, text: &mut String| {
            if !text.is_empty() {
                let node = AstNode::text(text.clone());
                text.clear();
                doc.children.push(node);
            }
        };

        while let Some(c) = chars.next() {
            match c {
                '#' if chars.peek() == Some(&'[') => {
                    flush_text(&mut document, &mut current_text);
                    chars.next();
                    let (section_content, found) = Self::read_until(&mut chars, ']');
                    if found {
                        let mut node = AstNode::new(NodeType::Section, section_content.clone());
                        for modifier in section_content.split(';') {
                            if let Some((key, value)) = modifier.split_once(':') {
                                node = node
                                    .with_attribute(key.trim(), Self::strip_quotes(value.trim()));
                            } else {
                                node = node.with_attribute("style", modifier.trim());
                            }
                        }
                        document = document.add_child(node);
                    } else {
                        current_text.push('#');
                        current_text.push('[');
                        current_text.push_str(&section_content);
                    }
                }
                '[' => {
                    flush_text(&mut document, &mut current_text);
                    let (bracket_content, found) = Self::read_until(&mut chars, ']');
                    if found {
                        if let Some(rest) = bracket_content.strip_prefix("break:") {
                            let break_value = Self::strip_quotes(rest.trim());
                            if Self::is_time_break(break_value) {
                                document = document.add_child(AstNode::new(
                                    NodeType::ShortBreak,
                                    format!("[{}]", break_value),
                                ));
                            } else {
                                let mut node =
                                    AstNode::new(NodeType::Break, break_value.to_string());
                                node = node.with_attribute("strength", break_value);
                                document = document.add_child(node);
                            }
                        } else if let Some(rest) = bracket_content.strip_prefix("mark:") {
                            let mark_value = Self::strip_quotes(rest.trim());
                            document = document
                                .add_child(AstNode::new(NodeType::Mark, mark_value.to_string()));
                        } else if Self::is_time_break(&bracket_content) {
                            document = document.add_child(AstNode::new(
                                NodeType::ShortBreak,
                                format!("[{}]", bracket_content),
                            ));
                        } else {
                            current_text.push('[');
                            current_text.push_str(&bracket_content);
                            current_text.push(']');
                        }
                    } else {
                        current_text.push('[');
                        current_text.push_str(&bracket_content);
                    }
                }
                '~' => {
                    let prev_is_boundary = current_text.is_empty()
                        || current_text.ends_with(|c: char| c.is_whitespace());
                    if !prev_is_boundary {
                        current_text.push('~');
                    } else {
                        flush_text(&mut document, &mut current_text);
                        let mut emphasized_text = String::new();
                        let mut found_end = false;
                        while let Some(&next_c) = chars.peek() {
                            chars.next();
                            if next_c == '~' {
                                found_end = true;
                                break;
                            }
                            emphasized_text.push(next_c);
                        }
                        if found_end
                            && !emphasized_text.is_empty()
                            && !emphasized_text.contains(' ')
                        {
                            document = document.add_child(AstNode::new(
                                NodeType::ShortEmphasisNone,
                                emphasized_text,
                            ));
                        } else {
                            current_text.push('~');
                            current_text.push_str(&emphasized_text);
                            if found_end {
                                current_text.push('~');
                            }
                        }
                    }
                }
                '-' => {
                    let prev_is_boundary = current_text.is_empty()
                        || current_text.ends_with(|c: char| c.is_whitespace());
                    if !prev_is_boundary {
                        current_text.push('-');
                    } else {
                        flush_text(&mut document, &mut current_text);
                        let mut emphasized_text = String::new();
                        let mut found_end = false;
                        while let Some(&next_c) = chars.peek() {
                            chars.next();
                            if next_c == '\n' || next_c == '\r' {
                                emphasized_text.push(next_c);
                                break;
                            }
                            if next_c == '-' {
                                let next_is_boundary =
                                    chars.peek().is_none_or(|c| c.is_whitespace());
                                if next_is_boundary {
                                    found_end = true;
                                    break;
                                } else {
                                    emphasized_text.push('-');
                                }
                            } else {
                                emphasized_text.push(next_c);
                            }
                        }
                        if found_end
                            && !emphasized_text.is_empty()
                            && !emphasized_text.contains(' ')
                        {
                            document = document.add_child(AstNode::new(
                                NodeType::ShortEmphasisReduced,
                                emphasized_text,
                            ));
                        } else {
                            current_text.push('-');
                            current_text.push_str(&emphasized_text);
                            if found_end {
                                current_text.push('-');
                            }
                        }
                    }
                }
                '+' => {
                    flush_text(&mut document, &mut current_text);
                    let mut plus_count = 1;
                    while chars.peek() == Some(&'+') {
                        chars.next();
                        plus_count += 1;
                    }
                    let mut emphasized_text = String::new();
                    let mut found_end = false;
                    while let Some(&next_c) = chars.peek() {
                        if next_c == '+' {
                            let mut closing_pluses = 0;
                            while chars.peek() == Some(&'+') {
                                chars.next();
                                closing_pluses += 1;
                            }
                            if closing_pluses == plus_count {
                                found_end = true;
                                break;
                            } else {
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
                        let node_type = if plus_count >= 2 {
                            NodeType::ShortEmphasisStrong
                        } else {
                            NodeType::ShortEmphasisModerate
                        };
                        document = document.add_child(AstNode::new(node_type, emphasized_text));
                    } else {
                        for _ in 0..plus_count {
                            current_text.push('+');
                        }
                        current_text.push_str(&emphasized_text);
                    }
                }
                '(' => {
                    flush_text(&mut document, &mut current_text);
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
                        if chars.peek() == Some(&'[') {
                            chars.next();
                            let (modifiers, found_bracket) = Self::read_until(&mut chars, ']');
                            if found_bracket {
                                let mut node =
                                    AstNode::new(NodeType::TextModifier, modifier_content);
                                for modifier in modifiers.split(';') {
                                    if let Some((key, value)) = modifier.split_once(':') {
                                        node = node.with_attribute(
                                            key.trim(),
                                            Self::strip_quotes(value.trim()),
                                        );
                                    } else {
                                        let key = modifier.trim();
                                        if !key.is_empty() {
                                            node = node.with_attribute(key, "");
                                        }
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
                        } else if chars.peek() == Some(&'{') {
                            chars.next();
                            let (alias_text, found_brace) = Self::read_until(&mut chars, '}');
                            if found_brace {
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
                        } else if chars.peek() == Some(&'/') {
                            chars.next();
                            let mut phoneme = String::new();
                            let mut found_slash = false;
                            while let Some(&next_c) = chars.peek() {
                                chars.next();
                                if next_c == '/' {
                                    found_slash = true;
                                    break;
                                }
                                phoneme.push(next_c);
                            }
                            if found_slash {
                                let mut node = AstNode::new(NodeType::ShortIpa, modifier_content);
                                node = node.with_attribute("phoneme", phoneme);
                                document = document.add_child(node);
                            } else {
                                current_text.push('(');
                                current_text.push_str(&modifier_content);
                                current_text.push(')');
                                current_text.push('/');
                                current_text.push_str(&phoneme);
                            }
                        } else {
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
                    flush_text(&mut document, &mut current_text);
                    let mut ipa_content = String::new();
                    let mut found_slash = false;
                    while let Some(&next_c) = chars.peek() {
                        if next_c == '/' {
                            chars.next();
                            found_slash = true;
                            break;
                        }
                        if next_c == ' ' || next_c == '\n' || next_c == '\r' || next_c == '\t' {
                            break;
                        }
                        chars.next();
                        ipa_content.push(next_c);
                    }
                    if found_slash && !ipa_content.is_empty() {
                        let mut node = AstNode::new(NodeType::BareIpa, "ipa".to_string());
                        node = node.with_attribute("alphabet", "ipa");
                        node = node.with_attribute("ph", ipa_content.trim().to_string());
                        document = document.add_child(node);
                    } else if found_slash {
                        current_text.push('/');
                        current_text.push('/');
                    } else {
                        current_text.push('/');
                        current_text.push_str(&ipa_content);
                    }
                }
                '{' => {
                    flush_text(&mut document, &mut current_text);
                    let (sub_text, found_brace) = Self::read_until(&mut chars, '}');
                    if found_brace && !sub_text.is_empty() {
                        let mut alias_text = String::new();
                        while let Some(&next_c) = chars.peek() {
                            if next_c.is_whitespace()
                                || next_c == '('
                                || next_c == '['
                                || next_c == '+'
                                || next_c == '~'
                                || next_c == '!'
                                || next_c == '/'
                                || next_c == '{'
                                || next_c == '}'
                                || next_c == '#'
                            {
                                break;
                            }
                            chars.next();
                            alias_text.push(next_c);
                        }
                        let mut node = AstNode::new(NodeType::ShortSub, sub_text);
                        if !alias_text.is_empty() {
                            node = node.with_attribute("alias", alias_text);
                        }
                        document = document.add_child(node);
                    } else {
                        current_text.push('{');
                        current_text.push_str(&sub_text);
                    }
                }
                '!' => {
                    if chars.peek() == Some(&'[') {
                        flush_text(&mut document, &mut current_text);
                        chars.next();
                        let (caption, found_caption_end) = Self::read_until(&mut chars, ']');

                        if found_caption_end && chars.peek() == Some(&'(') {
                            chars.next();
                            let (url, found_url_end) = Self::read_until(&mut chars, ')');
                            if found_url_end {
                                let mut node = AstNode::new(NodeType::Audio, caption);
                                node = node.with_attribute("src", Self::strip_quotes(&url));
                                document = document.add_child(node);
                            } else {
                                current_text.push_str(&format!("![{}]", caption));
                            }
                        } else if found_caption_end && chars.peek() == Some(&'[') {
                            chars.next();
                            let (url, found_url_end) = Self::read_until(&mut chars, ']');
                            if found_url_end {
                                let mut node = AstNode::new(NodeType::Audio, caption);
                                node = node.with_attribute("src", Self::strip_quotes(&url));
                                document = document.add_child(node);
                            } else {
                                current_text.push_str(&format!("![{}]", caption));
                            }
                        } else if found_caption_end {
                            let possible_url = Self::strip_quotes(&caption);
                            if possible_url.starts_with("http://")
                                || possible_url.starts_with("https://")
                                || possible_url.starts_with("soundbank://")
                                || possible_url.contains("://")
                                || possible_url.contains('.')
                            {
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
                        flush_text(&mut document, &mut current_text);
                        chars.next();
                        let (caption, found_caption_end) = Self::read_until(&mut chars, ')');
                        if found_caption_end && chars.peek() == Some(&'[') {
                            chars.next();
                            let (url, found_url_end) = Self::read_until(&mut chars, ']');
                            if found_url_end {
                                let mut node = AstNode::new(NodeType::Audio, caption);
                                node = node.with_attribute("src", Self::strip_quotes(&url));
                                document = document.add_child(node);
                            } else {
                                current_text.push_str(&format!("!({}[", caption));
                            }
                        } else {
                            current_text.push_str(&format!("!({}", caption));
                        }
                    } else {
                        current_text.push('!');
                    }
                }
                _ => {
                    current_text.push(c);
                }
            }
        }

        if !current_text.is_empty() {
            document = document.add_child(AstNode::text(current_text));
        }

        Ok(document)
    }

    fn strip_quotes(s: &str) -> &str {
        let s = s.trim();
        if s.len() >= 2 {
            let first = s.chars().next().unwrap();
            let last = s.chars().last().unwrap();
            if (first == '"' && last == '"') || (first == '\'' && last == '\'') {
                return &s[1..s.len() - 1];
            }
        }
        s
    }

    fn is_time_break(s: &str) -> bool {
        s.ends_with("s") || s.ends_with("ms")
    }

    fn read_until(chars: &mut std::iter::Peekable<std::str::Chars>, end: char) -> (String, bool) {
        let mut content = String::new();
        let mut found = false;
        while let Some(&next_c) = chars.peek() {
            chars.next();
            if next_c == end {
                found = true;
                break;
            }
            content.push(next_c);
        }
        (content, found)
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
        let result =
            SpeechMarkdownParser::to_ssml(input, crate::formatters::base::Platform::AmazonAlexa);
        println!("=== Emphasis SSML Debug ===");
        println!("Input: {}", input);
        println!("SSML Result: {:?}", result);
        println!("==========================");
    }

    #[test]
    fn test_is_speech_markdown() {
        assert!(!SpeechMarkdownParser::is_speech_markdown("Hello world"));
        assert!(!SpeechMarkdownParser::is_speech_markdown(""));
        assert!(SpeechMarkdownParser::is_speech_markdown("Hello (world)[emphasis:\"strong\"]"));
        assert!(SpeechMarkdownParser::is_speech_markdown("Sample [2s] text"));
        assert!(SpeechMarkdownParser::is_speech_markdown("++strong++"));
        assert!(SpeechMarkdownParser::is_speech_markdown("~word~"));
        assert!(SpeechMarkdownParser::is_speech_markdown("{Al}aluminum"));
        assert!(SpeechMarkdownParser::is_speech_markdown("![audio](url)"));
    }

    #[test]
    fn test_validate() {
        assert!(SpeechMarkdownParser::validate("Hello world").is_ok());
        assert!(SpeechMarkdownParser::validate("Hello (world)[emphasis:\"strong\"]").is_ok());
        assert!(SpeechMarkdownParser::validate("Sample [2s] text").is_ok());
        assert!(SpeechMarkdownParser::validate("++strong++").is_ok());
    }
}
