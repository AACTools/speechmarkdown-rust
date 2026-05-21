use crate::ast::{AstNode, NodeType};
use crate::error::Result;
use crate::formatters::base::{Formatter, FormatterOptions};
use std::collections::HashMap;

pub struct SsmlFormatterBase {
    options: FormatterOptions,
    modifier_mappings: HashMap<String, String>,
    tag_sort_order: Vec<String>,
}

impl SsmlFormatterBase {
    pub fn new(options: FormatterOptions) -> Self {
        let modifier_mappings = Self::create_default_mappings();
        let tag_sort_order = Self::create_default_tag_order();

        Self {
            options,
            modifier_mappings,
            tag_sort_order,
        }
    }

    fn create_default_mappings() -> HashMap<String, String> {
        let mut mappings = HashMap::new();

        // Standard SSML mappings
        mappings.insert("emphasis".to_string(), "emphasis".to_string());
        mappings.insert("voice".to_string(), "voice".to_string());
        mappings.insert("lang".to_string(), "lang".to_string());
        mappings.insert("rate".to_string(), "prosody".to_string());
        mappings.insert("pitch".to_string(), "prosody".to_string());
        mappings.insert("volume".to_string(), "prosody".to_string());
        mappings.insert("whisper".to_string(), "amazon:effect".to_string());

        mappings
    }

    fn create_default_tag_order() -> Vec<String> {
        vec![
            "emphasis".to_string(),
            "say-as".to_string(),
            "prosody".to_string(),
            "voice".to_string(),
            "lang".to_string(),
            "sub".to_string(),
            "phoneme".to_string(),
        ]
    }

    /// Format an AST node with proper SSML tag application
    fn format_node_with_tags(&self, node: &AstNode) -> Result<String> {
        match node.node_type {
            // Structural nodes
            NodeType::Document => self.format_document(node),
            NodeType::Paragraph => self.format_paragraph(node),
            NodeType::SimpleLine => self.format_simple_line(node),
            NodeType::EmptyLine => self.format_empty_line(node),
            NodeType::Section => self.format_section(node),

            // Content nodes
            NodeType::PlainText => Ok(self.escape_xml(&node.text)),

            // Breaks
            NodeType::ShortBreak => self.format_short_break(node),
            NodeType::Break => self.format_break(node),

            // Emphasis
            NodeType::ShortEmphasisModerate => self.format_emphasis(node, "moderate"),
            NodeType::ShortEmphasisStrong => self.format_emphasis(node, "strong"),
            NodeType::ShortEmphasisNone => self.format_emphasis(node, "reduced"),
            NodeType::ShortEmphasisReduced => self.format_emphasis(node, "reduced"),

            // Text modifiers
            NodeType::TextModifier => self.format_text_modifier(node),

            // Audio
            NodeType::Audio => self.format_audio(node),

            // Mark tags
            NodeType::Mark => self.format_mark(node),

            // IPA pronunciation
            NodeType::ShortIpa => self.format_ipa(node),
            NodeType::BareIpa => self.format_bare_ipa(node),

            // Substitution
            NodeType::ShortSub => self.format_short_sub(node),

            // Default: handle as plain text
            _ => Ok(self.escape_xml(&node.text)),
        }
    }

    fn format_document(&self, node: &AstNode) -> Result<String> {
        let mut content = String::new();
        let mut children_iter = node.children.iter().peekable();

        while let Some(child) = children_iter.next() {
            if child.node_type == NodeType::Section {
                // Collect all content until the next section or end
                let mut section_content = String::new();

                while let Some(next_child) = children_iter.peek() {
                    if next_child.node_type == NodeType::Section {
                        break;
                    }
                    let next_child = children_iter.next().unwrap();
                    section_content.push_str(&self.format_node_with_tags(next_child)?);
                }

                // Format the section with its content
                let section_open = self.format_node_with_tags(child)?;
                let section_close = if child.node_type == NodeType::Section {
                    self.format_section_close(child)?
                } else {
                    String::new()
                };

                content.push_str(&section_open);
                content.push_str(&section_content);
                content.push_str(&section_close);
            } else {
                content.push_str(&self.format_node_with_tags(child)?);
            }
        }

        if self.options.include_speak_tag {
            Ok(format!("<speak>\n{}\n</speak>", content))
        } else {
            Ok(content)
        }
    }

    fn format_paragraph(&self, node: &AstNode) -> Result<String> {
        let mut content = String::new();

        for child in &node.children {
            content.push_str(&self.format_node_with_tags(child)?);
        }

        if self.options.include_paragraph_tag {
            Ok(format!("<p>{}</p>", content))
        } else {
            Ok(content)
        }
    }

    fn format_simple_line(&self, node: &AstNode) -> Result<String> {
        let mut content = String::new();

        for child in &node.children {
            content.push_str(&self.format_node_with_tags(child)?);
        }

        Ok(content)
    }

    fn format_empty_line(&self, _node: &AstNode) -> Result<String> {
        if self.options.preserve_empty_lines {
            Ok("\n".to_string())
        } else {
            Ok(String::new())
        }
    }

    fn format_section(&self, node: &AstNode) -> Result<String> {
        let mut tags: Vec<(String, HashMap<String, String>)> = Vec::new();
        for (key, value) in &node.attributes {
            if let Some(tag_info) = self.attribute_to_tag(key, value) {
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

        let mut result = String::new();
        for (tag_name, attrs) in &tags {
            let attr_string = attrs
                .iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, self.escape_xml(v)))
                .collect::<Vec<_>>()
                .join(" ");
            if attr_string.is_empty() {
                result.push_str(&format!("<{}>\n", tag_name));
            } else {
                result.push_str(&format!("<{} {}>\n", tag_name, attr_string));
            }
        }
        Ok(result)
    }

    fn format_section_close(&self, node: &AstNode) -> Result<String> {
        let mut tags: Vec<(String, HashMap<String, String>)> = Vec::new();
        for (key, value) in &node.attributes {
            if let Some(tag_info) = self.attribute_to_tag(key, value) {
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

        let mut result = String::new();
        for (tag_name, _) in tags.iter().rev() {
            result.push_str(&format!("</{}>\n", tag_name));
        }
        Ok(result)
    }

    fn format_short_break(&self, node: &AstNode) -> Result<String> {
        // Extract time from text like [2s] or [250ms]
        let time = node.text.trim_start_matches('[').trim_end_matches(']');
        Ok(format!("<break time=\"{}\"/>", time))
    }

    fn format_break(&self, node: &AstNode) -> Result<String> {
        // Get strength from attributes or use the text directly
        let strength = node
            .attributes
            .get("strength")
            .unwrap_or(&node.text)
            .clone();

        Ok(format!("<break strength=\"{}\"/>", strength))
    }

    fn format_emphasis(&self, node: &AstNode, level: &str) -> Result<String> {
        Ok(format!(
            "<emphasis level=\"{}\">{}</emphasis>",
            level,
            self.escape_xml(&node.text)
        ))
    }

    fn format_text_modifier(&self, node: &AstNode) -> Result<String> {
        let mut tags: Vec<(String, HashMap<String, String>)> = Vec::new();

        for (key, value) in &node.attributes {
            if let Some(tag_info) = self.attribute_to_tag(key, value) {
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
            return Ok(self.escape_xml(&node.text));
        }

        self.apply_tags_to_text(&node.text, &tags)
    }

    fn format_audio(&self, node: &AstNode) -> Result<String> {
        let src = node.attributes.get("src").unwrap_or(&String::new()).clone();

        let caption = &node.text;

        if caption.is_empty() {
            Ok(format!("<audio src=\"{}\"/>", src))
        } else {
            Ok(format!(
                "<audio src=\"{}\">\n<desc>{}</desc>\n</audio>",
                src,
                self.escape_xml(caption)
            ))
        }
    }

    fn format_mark(&self, node: &AstNode) -> Result<String> {
        Ok(format!("<mark name=\"{}\"/>", self.escape_xml(&node.text)))
    }

    fn format_ipa(&self, node: &AstNode) -> Result<String> {
        let phoneme = node
            .attributes
            .get("phoneme")
            .unwrap_or(&String::new())
            .clone();

        if phoneme.is_empty() {
            Ok(self.escape_xml(&node.text))
        } else {
            Ok(format!(
                "<phoneme alphabet=\"ipa\" ph=\"{}\">{}</phoneme>",
                self.escape_xml(&phoneme),
                self.escape_xml(&node.text)
            ))
        }
    }

    fn format_bare_ipa(&self, node: &AstNode) -> Result<String> {
        let phoneme = node.attributes.get("ph").unwrap_or(&node.text).clone();

        Ok(format!(
            "<phoneme alphabet=\"ipa\" ph=\"{}\">ipa</phoneme>",
            self.escape_xml(&phoneme)
        ))
    }

    fn format_short_sub(&self, node: &AstNode) -> Result<String> {
        let alias = node
            .attributes
            .get("alias")
            .unwrap_or(&String::new())
            .clone();

        if alias.is_empty() {
            Ok(self.escape_xml(&node.text))
        } else {
            Ok(format!(
                "<sub alias=\"{}\">{}</sub>",
                self.escape_xml(&alias),
                self.escape_xml(&node.text)
            ))
        }
    }

    /// Convert a modifier node to SSML tag information
    fn modifier_to_tag(&self, node: &AstNode) -> Option<(String, HashMap<String, String>)> {
        // First check the mappings
        if let Some(tag_name) = self.modifier_mappings.get(&node.text.to_lowercase()) {
            return self.extract_tag_attributes(node, tag_name);
        }

        // If not in mappings, determine based on node type
        let tag_name = match node.node_type {
            NodeType::Voice => "voice",
            NodeType::Lang => "lang",
            NodeType::Rate => "prosody",
            NodeType::Pitch => "prosody",
            NodeType::Volume => "prosody",
            NodeType::Emphasis => "emphasis",
            NodeType::Whisper => "amazon:effect",
            _ => return None,
        };

        self.extract_tag_attributes(node, tag_name)
    }

    fn extract_tag_attributes(
        &self,
        node: &AstNode,
        tag_name: &str,
    ) -> Option<(String, HashMap<String, String>)> {
        let mut attributes = HashMap::new();

        // Extract attribute value if present
        if let Some(value) = node.attributes.get("value") {
            attributes.insert(self.get_attribute_name(tag_name), value.clone());
        }

        // Set default attribute names
        match tag_name {
            "voice" => {
                if !attributes.contains_key("name") {
                    attributes.insert("name".to_string(), node.text.clone());
                }
            }
            "lang" => {
                if !attributes.contains_key("xml:lang") {
                    attributes.insert("xml:lang".to_string(), node.text.clone());
                }
            }
            "prosody" => {
                if !attributes.is_empty() {
                    // Determine which prosody attribute based on node type
                    let attr_name = match node.node_type {
                        NodeType::Rate => "rate",
                        NodeType::Pitch => "pitch",
                        NodeType::Volume => "volume",
                        _ => "rate",
                    };
                    if !attributes.contains_key(attr_name) {
                        attributes.insert(attr_name.to_string(), node.text.clone());
                    }
                }
            }
            _ => {}
        }

        Some((tag_name.to_string(), attributes))
    }

    fn get_attribute_name(&self, tag: &str) -> String {
        match tag {
            "voice" => "name".to_string(),
            "lang" => "xml:lang".to_string(),
            "prosody" => "rate".to_string(),
            "emphasis" => "level".to_string(),
            _ => "value".to_string(),
        }
    }

    fn attribute_to_tag(
        &self,
        key: &str,
        value: &str,
    ) -> Option<(String, HashMap<String, String>)> {
        let mut attributes = HashMap::new();

        match key.to_lowercase().as_str() {
            "address" => Some(("say-as".to_string(), {
                let mut attrs = HashMap::new();
                attrs.insert("interpret-as".to_string(), "address".to_string());
                attrs
            })),
            "date" => Some(("say-as".to_string(), {
                let mut attrs = HashMap::new();
                attrs.insert("interpret-as".to_string(), "date".to_string());
                if !value.is_empty() {
                    attrs.insert("format".to_string(), value.to_string());
                }
                attrs
            })),
            "time" => Some(("say-as".to_string(), {
                let mut attrs = HashMap::new();
                attrs.insert("interpret-as".to_string(), "time".to_string());
                if !value.is_empty() {
                    attrs.insert("format".to_string(), value.to_string());
                }
                attrs
            })),
            "number" | "cardinal" => Some(("say-as".to_string(), {
                let mut attrs = HashMap::new();
                attrs.insert("interpret-as".to_string(), "number".to_string());
                attrs
            })),
            "ordinal" => Some(("say-as".to_string(), {
                let mut attrs = HashMap::new();
                attrs.insert("interpret-as".to_string(), "ordinal".to_string());
                attrs
            })),
            "characters" | "chars" | "digits" | "drc" => Some(("say-as".to_string(), {
                let mut attrs = HashMap::new();
                attrs.insert("interpret-as".to_string(), "characters".to_string());
                attrs
            })),
            "fraction" => Some(("say-as".to_string(), {
                let mut attrs = HashMap::new();
                attrs.insert("interpret-as".to_string(), "fraction".to_string());
                attrs
            })),
            "unit" => Some(("say-as".to_string(), {
                let mut attrs = HashMap::new();
                attrs.insert("interpret-as".to_string(), "unit".to_string());
                attrs
            })),
            "interjection" => Some(("say-as".to_string(), {
                let mut attrs = HashMap::new();
                attrs.insert("interpret-as".to_string(), "interjection".to_string());
                attrs
            })),
            "expletive" | "bleep" => Some(("say-as".to_string(), {
                let mut attrs = HashMap::new();
                attrs.insert("interpret-as".to_string(), "expletive".to_string());
                attrs
            })),
            "telephone" | "phone" => Some(("say-as".to_string(), {
                let mut attrs = HashMap::new();
                attrs.insert("interpret-as".to_string(), "telephone".to_string());
                attrs
            })),
            "ipa" => Some(("phoneme".to_string(), {
                let mut attrs = HashMap::new();
                attrs.insert("alphabet".to_string(), "ipa".to_string());
                if !value.is_empty() {
                    attrs.insert("ph".to_string(), value.to_string());
                }
                attrs
            })),
            "sub" => {
                if !value.is_empty() {
                    attributes.insert("alias".to_string(), value.to_string());
                }
                Some(("sub".to_string(), attributes))
            }
            "voice" => {
                if !value.is_empty() {
                    attributes.insert("name".to_string(), value.to_string());
                }
                Some(("voice".to_string(), attributes))
            }
            "lang" => {
                if !value.is_empty() {
                    attributes.insert("xml:lang".to_string(), value.to_string());
                }
                Some(("lang".to_string(), attributes))
            }
            "rate" => {
                let rate_val = if value.is_empty() { "medium" } else { value };
                attributes.insert("rate".to_string(), rate_val.to_string());
                Some(("prosody".to_string(), attributes))
            }
            "pitch" => {
                let pitch_val = if value.is_empty() { "medium" } else { value };
                attributes.insert("pitch".to_string(), pitch_val.to_string());
                Some(("prosody".to_string(), attributes))
            }
            "volume" | "vol" => {
                let vol_val = if value.is_empty() { "medium" } else { value };
                attributes.insert("volume".to_string(), vol_val.to_string());
                Some(("prosody".to_string(), attributes))
            }
            "timbre" => {
                let timbre_val = if value.is_empty() { "medium" } else { value };
                attributes.insert("pitch".to_string(), timbre_val.to_string());
                Some(("prosody".to_string(), attributes))
            }
            "emphasis" => {
                let level = if value.is_empty() { "moderate" } else { value };
                attributes.insert("level".to_string(), level.to_string());
                Some(("emphasis".to_string(), attributes))
            }
            "whisper" => Some(("amazon:effect".to_string(), {
                let mut attrs = HashMap::new();
                attrs.insert("name".to_string(), "whispered".to_string());
                attrs
            })),
            _ => None,
        }
    }

    fn handle_special_modifiers(
        &self,
        node: &AstNode,
    ) -> Option<(String, HashMap<String, String>)> {
        // Handle special cases based on the modifier keys in attributes
        for key in node.attributes.keys() {
            if let Some(tag_info) = self.attribute_to_tag(key, "") {
                return Some(tag_info);
            }
        }
        None
    }

    /// Apply multiple tags to text in the correct order
    fn apply_tags_to_text(
        &self,
        text: &str,
        tags: &[(String, HashMap<String, String>)],
    ) -> Result<String> {
        let mut current_text = text.to_string();

        // Sort tags according to the defined order
        let mut sorted_tags = tags.to_vec();
        sorted_tags.sort_by_key(|(tag_name, _)| {
            self.tag_sort_order
                .iter()
                .position(|t| t == tag_name)
                .unwrap_or(usize::MAX)
        });

        // Apply tags from inside to outside (reverse order)
        for (tag_name, attributes) in sorted_tags.iter().rev() {
            let attr_string = attributes
                .iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, self.escape_xml(v)))
                .collect::<Vec<_>>()
                .join(" ");

            if attr_string.is_empty() {
                current_text = format!("<{}>{}</{}>", tag_name, current_text, tag_name);
            } else {
                current_text = format!(
                    "<{} {}>{}</{}>",
                    tag_name, attr_string, current_text, tag_name
                );
            }
        }

        Ok(current_text)
    }

    fn format_children_with_modifiers(
        &self,
        _node: &AstNode,
        children: &[AstNode],
    ) -> Result<String> {
        let mut content = String::new();

        for child in children {
            content.push_str(&self.format_node_with_tags(child)?);
        }

        Ok(content)
    }

    /// Escape XML special characters
    pub fn escape_xml(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
        // Note: We don't escape apostrophes as they're generally safe in SSML
        // and many implementations prefer literal ' over &apos;
    }
}

impl Formatter for SsmlFormatterBase {
    fn format(&self, ast: &AstNode) -> Result<String> {
        self.format_node_with_tags(ast)
    }

    fn format_node(&self, node: &AstNode) -> Result<String> {
        self.format_node_with_tags(node)
    }
}
