use crate::ast::{AstNode, NodeType};
use crate::error::Result;
use crate::formatters::base::{Formatter, FormatterOptions};
use crate::formatters::ssml::base::{
    attrs_merge, format_attr_string_ordered, SsmlFormatterBase, TagAttrs, TagInfo,
};

const AZURE_EXPRESS_AS_STYLES: &[&str] = &[
    "angry",
    "cheerful",
    "excited",
    "friendly",
    "hopeful",
    "sad",
    "shouting",
    "terrified",
    "whispering",
    "unfriendly",
    "depressed",
    "serious",
    "calm",
    "fearful",
    "envious",
    "gentle",
    "lyrical",
    "narration-professional",
    "narration-relaxed",
    "newscast-casual",
    "newscast-formal",
    "chat",
    "customerservice",
    "empathetic",
    "documentary-narration",
    "advertisement_upbeat",
    "sports_commentary",
    "sports_commentary_excited",
    "poetry-reading",
    "assistant",
    "embarrassed",
    "disgruntled",
];

pub fn azure_voice_name(name: &str) -> String {
    let lower = name.to_lowercase();
    let mapping: [(&str, &str); 11] = [
        ("jenny", "en-US-JennyNeural"),
        ("guy", "en-US-GuyNeural"),
        ("aria", "en-US-AriaNeural"),
        ("davis", "en-US-DavisNeural"),
        ("amber", "en-US-AmberNeural"),
        ("ana", "en-US-AnaNeural"),
        ("andrew", "en-US-AndrewNeural"),
        ("emma", "en-US-EmmaNeural"),
        ("brian", "en-US-BrianNeural"),
        ("christopher", "en-US-ChristopherNeural"),
        ("eric", "en-US-EricNeural"),
    ];
    for (key, neural) in &mapping {
        if lower == *key {
            return neural.to_string();
        }
    }
    let mut result = String::new();
    let mut cap_next = true;
    for c in name.chars() {
        if c == '-' || c == '_' || c == ' ' {
            result.push('-');
            cap_next = true;
        } else if cap_next {
            for uc in c.to_uppercase() {
                result.push(uc);
            }
            cap_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

pub struct MicrosoftAzureSsmlFormatter {
    base: SsmlFormatterBase,
    options: FormatterOptions,
}

impl MicrosoftAzureSsmlFormatter {
    pub fn new(options: FormatterOptions) -> Self {
        let base = SsmlFormatterBase::new(options.clone());
        Self { base, options }
    }

    fn is_valid_azure_style(style: &str) -> bool {
        AZURE_EXPRESS_AS_STYLES.contains(&style)
    }

    fn section_style(node: &AstNode) -> Option<String> {
        node.attributes
            .get("style")
            .or_else(|| node.attributes.get("emotion"))
            .cloned()
    }

    fn is_azure_express_section(node: &AstNode) -> bool {
        let style = Self::section_style(node);
        match style {
            Some(s) => {
                if matches!(s.as_str(), "voice" | "lang" | "device" | "defaults") {
                    return false;
                }
                Self::is_valid_azure_style(&s)
            }
            None => false,
        }
    }

    fn is_unsupported_emotion_section(node: &AstNode) -> bool {
        let style = Self::section_style(node);
        match style {
            Some(s) => {
                if matches!(s.as_str(), "voice" | "lang" | "device" | "defaults") {
                    return false;
                }
                !Self::is_valid_azure_style(&s)
            }
            None => {
                for key in &node.attribute_keys {
                    if key == "disappointed" || key == "excited" {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn is_emotion_section(node: &AstNode) -> bool {
        let style = Self::section_style(node);
        match style {
            Some(s) => !matches!(s.as_str(), "voice" | "lang" | "device"),
            None => {
                for key in &node.attribute_keys {
                    if key == "disappointed" || key == "excited" {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn azure_attribute_to_tag(&self, key: &str, value: &str) -> Option<TagInfo> {
        let mut attributes: TagAttrs = Vec::new();
        match key.to_lowercase().as_str() {
            "emphasis" => None,
            "whisper" => {
                attributes.push(("volume".to_string(), "x-soft".to_string()));
                attributes.push(("rate".to_string(), "slow".to_string()));
                Some(("prosody".to_string(), attributes))
            }
            "number" | "cardinal" => Some(("say-as".to_string(), {
                let mut attrs: TagAttrs = Vec::new();
                attrs.push(("interpret-as".to_string(), "cardinal".to_string()));
                attrs
            })),
            "excited" | "disappointed" => Some(("mstts:express-as".to_string(), {
                let mut attrs: TagAttrs = Vec::new();
                attrs.push(("style".to_string(), key.to_lowercase()));
                attrs
            })),
            "voice" => {
                if value.is_empty() || value == "device" {
                    return None;
                }
                let neural_name = azure_voice_name(value);
                attributes.push(("name".to_string(), neural_name));
                Some(("voice".to_string(), attributes))
            }
            _ => self.base.attribute_to_tag(key, value),
        }
    }

    fn format_azure_text_modifier(&self, node: &AstNode) -> Result<String> {
        let mut tags: Vec<TagInfo> = Vec::new();
        let mut last_say_as: Option<TagInfo> = None;

        for key in &node.attribute_keys {
            let value = match node.attributes.get(key) {
                Some(v) => v,
                None => continue,
            };
            if let Some(tag_info) = self.azure_attribute_to_tag(key, value) {
                let tag_name = tag_info.0.clone();
                if tag_name == "prosody" {
                    if let Some(existing) = tags.iter_mut().find(|(name, _)| name == "prosody") {
                        attrs_merge(&mut existing.1, tag_info.1);
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

    fn format_azure_node(&self, node: &AstNode) -> Result<String> {
        match node.node_type {
            NodeType::PlainText => Ok(node.text.clone()),
            NodeType::TextModifier => self.format_azure_text_modifier(node),
            _ => self.base.format_node_internal(node),
        }
    }

    fn format_azure_section(&self, node: &AstNode) -> Result<String> {
        let mut tags: Vec<TagInfo> = Vec::new();

        if let Some(style) = node.attributes.get("style") {
            if style != "defaults" {
                if let Some(tag_info) = self.azure_attribute_to_tag(style, "") {
                    tags.push(tag_info);
                }
            }
        }

        for key in &node.attribute_keys {
            let value = match node.attributes.get(key) {
                Some(v) => v,
                None => continue,
            };
            if key == "style" {
                continue;
            }
            if let Some(tag_info) = self.azure_attribute_to_tag(key, value) {
                let tag_name = tag_info.0.clone();
                if tag_name == "prosody" {
                    if let Some(existing) = tags.iter_mut().find(|(name, _)| name == "prosody") {
                        attrs_merge(&mut existing.1, tag_info.1);
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
        for (i, (tag_name, attrs)) in tags.iter().enumerate() {
            let attr_string = format_attr_string_ordered(tag_name, attrs);
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

    fn format_azure_section_close(&self, node: &AstNode) -> Result<String> {
        let mut tags: Vec<TagInfo> = Vec::new();

        if let Some(style) = node.attributes.get("style") {
            if style != "defaults" {
                if let Some(tag_info) = self.azure_attribute_to_tag(style, "") {
                    tags.push(tag_info);
                }
            }
        }

        for key in &node.attribute_keys {
            let value = match node.attributes.get(key) {
                Some(v) => v,
                None => continue,
            };
            if key == "style" {
                continue;
            }
            if let Some(tag_info) = self.azure_attribute_to_tag(key, value) {
                let tag_name = tag_info.0.clone();
                if tag_name == "prosody" {
                    if let Some(existing) = tags.iter_mut().find(|(name, _)| name == "prosody") {
                        attrs_merge(&mut existing.1, tag_info.1);
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
        for (i, (tag_name, _)) in tags.iter().rev().enumerate() {
            result.push_str(&format!("</{}>", tag_name));
            if i < tags.len() - 1 {
                result.push('\n');
            }
        }
        Ok(result)
    }

    fn has_unsupported_emotion_sections(ast: &AstNode) -> bool {
        for child in &ast.children {
            if child.node_type == NodeType::Section && Self::is_unsupported_emotion_section(child) {
                return true;
            }
        }
        false
    }

    fn format_document_sections(&self, ast: &AstNode) -> Result<String> {
        let passthrough_emotions = Self::has_unsupported_emotion_sections(ast);
        let mut content = String::new();
        let mut children_iter = ast.children.iter().peekable();

        while let Some(child) = children_iter.next() {
            if child.node_type == NodeType::Section {
                let is_express = !passthrough_emotions && Self::is_azure_express_section(child);
                let is_unsupported = Self::is_unsupported_emotion_section(child);
                let is_emotion_passthrough =
                    passthrough_emotions && Self::is_emotion_section(child);
                let is_defaults = child
                    .attributes
                    .get("style")
                    .map_or(false, |s| s == "defaults");

                let mut section_content_raw = String::new();
                while let Some(next_child) = children_iter.peek() {
                    if next_child.node_type == NodeType::Section {
                        break;
                    }
                    let next_child = children_iter.next().unwrap();
                    section_content_raw.push_str(&self.format_azure_node(next_child)?);
                }

                if is_unsupported || is_emotion_passthrough || is_defaults {
                    content.push_str(&format!("#[{}]", child.text));
                    content.push_str(&section_content_raw);
                } else if is_express {
                    let style = Self::section_style(child).unwrap_or_default();
                    let had_leading_newline = section_content_raw.starts_with('\n');
                    let section_content = if had_leading_newline {
                        &section_content_raw[1..]
                    } else {
                        &section_content_raw
                    };

                    content.push_str(&format!("<mstts:express-as style=\"{}\">", style));
                    if had_leading_newline {
                        content.push('\n');
                    }
                    content.push_str(section_content);
                    content.push_str("</mstts:express-as>");
                    if had_leading_newline {
                        content.push('\n');
                    }
                } else {
                    let had_leading_newline = section_content_raw.starts_with('\n');
                    let section_content = if had_leading_newline {
                        &section_content_raw[1..]
                    } else {
                        &section_content_raw
                    };

                    let section_open = self.format_azure_section(child)?;
                    let section_close = if !section_open.is_empty() {
                        self.format_azure_section_close(child)?
                    } else {
                        String::new()
                    };

                    if !section_open.is_empty() {
                        content.push_str(&section_open);
                        if had_leading_newline {
                            content.push('\n');
                        }
                        content.push_str(section_content);
                        content.push_str(&section_close);
                        if had_leading_newline {
                            content.push('\n');
                        }
                    } else {
                        content.push_str(section_content);
                    }
                }
            } else {
                content.push_str(&self.format_azure_node(child)?);
            }
        }

        Ok(content)
    }
}

impl Formatter for MicrosoftAzureSsmlFormatter {
    fn format(&self, ast: &AstNode) -> Result<String> {
        let content = self.format_document_sections(ast)?;

        if self.options.include_speak_tag {
            let trimmed = content.trim_end_matches('\n');
            let use_mstts = trimmed.contains("mstts:express-as");
            if use_mstts {
                Ok(format!(
                    "<speak xmlns:mstts=\"https://www.w3.org/2001/mstts\">\n{}\n</speak>",
                    trimmed
                ))
            } else {
                Ok(format!("<speak>\n{}\n</speak>", trimmed))
            }
        } else {
            Ok(content)
        }
    }

    fn format_node(&self, node: &AstNode) -> Result<String> {
        self.format_azure_node(node)
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
