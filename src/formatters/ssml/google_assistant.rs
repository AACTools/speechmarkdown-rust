use std::collections::HashMap;

use crate::ast::{AstNode, NodeType};
use crate::error::Result;
use crate::formatters::base::{Formatter, FormatterOptions};
use crate::formatters::ssml::base::SsmlFormatterBase;

pub struct GoogleAssistantSsmlFormatter {
    base: SsmlFormatterBase,
    options: FormatterOptions,
}

impl GoogleAssistantSsmlFormatter {
    pub fn new(options: FormatterOptions) -> Self {
        let base = SsmlFormatterBase::new(options.clone());
        Self { base, options }
    }

    fn google_attribute_to_tag(
        &self,
        key: &str,
        value: &str,
    ) -> Option<(String, HashMap<String, String>)> {
        let mut attributes = HashMap::new();
        match key.to_lowercase().as_str() {
            "whisper" => {
                attributes.insert("volume".to_string(), "x-soft".to_string());
                attributes.insert("rate".to_string(), "slow".to_string());
                Some(("prosody".to_string(), attributes))
            }
            "excited" | "disappointed" => None,
            "voice" | "lang" => None,
            "style" => {
                if !value.is_empty() {
                    attributes.insert("name".to_string(), value.to_string());
                }
                Some(("google:style".to_string(), attributes))
            }
            _ => self.base.attribute_to_tag(key, value),
        }
    }

    fn format_google_text_modifier(&self, node: &AstNode) -> Result<String> {
        let mut tags: Vec<(String, HashMap<String, String>)> = Vec::new();
        let mut last_say_as: Option<(String, HashMap<String, String>)> = None;

        for key in &node.attribute_keys {
            let value = match node.attributes.get(key) {
                Some(v) => v,
                None => continue,
            };
            if let Some(tag_info) = self.google_attribute_to_tag(key, value) {
                let tag_name = tag_info.0.clone();
                if tag_name == "prosody" {
                    if let Some(existing) = tags.iter_mut().find(|(name, _)| name == "prosody") {
                        for (k, v) in tag_info.1 {
                            existing.1.insert(k, v);
                        }
                        continue;
                    }
                }
                if tag_name == "say-as" {
                    last_say_as = Some(tag_info);
                    continue;
                }
                tags.push(tag_info);
            }
        }

        if let Some(say_as) = last_say_as {
            tags.push(say_as);
        }

        if tags.is_empty() {
            return Ok(node.text.clone());
        }

        self.base.apply_tags_to_text(&node.text, &tags)
    }

    fn format_google_section(&self, node: &AstNode) -> Result<String> {
        let mut tags: Vec<(String, HashMap<String, String>)> = Vec::new();

        for key in &node.attribute_keys {
            let value = match node.attributes.get(key) {
                Some(v) => v,
                None => continue,
            };
            if let Some(tag_info) = self.google_attribute_to_tag(key, value) {
                let tag_name = tag_info.0.clone();
                if tag_name == "prosody" {
                    if let Some(existing) = tags.iter_mut().find(|(name, _)| name == "prosody") {
                        for (k, v) in tag_info.1 {
                            existing.1.insert(k, v);
                        }
                        continue;
                    }
                }
                tags.push(tag_info);
            }
        }

        if tags.is_empty() {
            return Ok(String::new());
        }

        let section_tag_order = vec!["voice", "lang", "prosody", "emphasis"];
        tags.sort_by_key(|(tag_name, _)| {
            section_tag_order
                .iter()
                .position(|t| t == tag_name)
                .unwrap_or(usize::MAX)
        });

        let mut result = String::new();
        for (i, (tag_name, attrs)) in tags.iter().enumerate() {
            let attr_string = self.base.format_attr_string(tag_name, attrs);
            if i > 0 {
                result.push('\n');
            }
            if attr_string.is_empty() {
                result.push_str(&format!("<{}>", tag_name));
            } else {
                result.push_str(&format!("<{} {}>", tag_name, attr_string));
            }
        }
        Ok(result)
    }

    fn format_google_section_close(&self, node: &AstNode) -> Result<String> {
        let mut tags: Vec<(String, HashMap<String, String>)> = Vec::new();

        for key in &node.attribute_keys {
            let value = match node.attributes.get(key) {
                Some(v) => v,
                None => continue,
            };
            if let Some(tag_info) = self.google_attribute_to_tag(key, value) {
                let tag_name = tag_info.0.clone();
                if tag_name == "prosody" {
                    if let Some(existing) = tags.iter_mut().find(|(name, _)| name == "prosody") {
                        for (k, v) in tag_info.1 {
                            existing.1.insert(k, v);
                        }
                        continue;
                    }
                }
                tags.push(tag_info);
            }
        }

        let section_tag_order = vec!["voice", "lang", "prosody", "emphasis"];
        tags.sort_by_key(|(tag_name, _)| {
            section_tag_order
                .iter()
                .position(|t| t == tag_name)
                .unwrap_or(usize::MAX)
        });

        if tags.is_empty() {
            return Ok(String::new());
        }

        let mut result = String::new();
        for (i, (tag_name, _)) in tags.iter().rev().enumerate() {
            result.push_str(&format!("</{}>", tag_name));
            if i < tags.len() - 1 {
                result.push('\n');
            }
        }
        Ok(result)
    }
}

impl Formatter for GoogleAssistantSsmlFormatter {
    fn format(&self, ast: &AstNode) -> Result<String> {
        let mut content = String::new();
        let mut children_iter = ast.children.iter().peekable();

        while let Some(child) = children_iter.next() {
            if child.node_type == NodeType::Section {
                let mut section_content_raw = String::new();
                while let Some(next_child) = children_iter.peek() {
                    if next_child.node_type == NodeType::Section {
                        break;
                    }
                    let next_child = children_iter.next().unwrap();
                    section_content_raw.push_str(&self.format_google_node(next_child)?);
                }
                let section_content = if section_content_raw.starts_with('\n') {
                    &section_content_raw[1..]
                } else {
                    &section_content_raw
                };

                let section_open = self.format_google_section(child)?;
                let section_close = if !section_open.is_empty() {
                    self.format_google_section_close(child)?
                } else {
                    String::new()
                };

                content.push_str(&section_open);
                if !section_open.is_empty() && section_content.starts_with('\n') {
                    content.push('\n');
                }
                let final_content = if section_open.is_empty() {
                    section_content.trim_start()
                } else {
                    section_content
                };
                content.push_str(final_content);
                content.push_str(&section_close);
            } else {
                content.push_str(&self.format_google_node(child)?);
            }
        }

        if self.options.include_speak_tag {
            let content = content.trim_end_matches('\n');
            Ok(format!("<speak>\n{}\n</speak>", content))
        } else {
            Ok(content)
        }
    }

    fn format_node(&self, node: &AstNode) -> Result<String> {
        self.format_google_node(node)
    }
}

impl GoogleAssistantSsmlFormatter {
    fn format_google_node(&self, node: &AstNode) -> Result<String> {
        match node.node_type {
            NodeType::PlainText => Ok(node.text.clone()),
            NodeType::TextModifier => self.format_google_text_modifier(node),
            _ => self.base.format_node_internal(node),
        }
    }
}
