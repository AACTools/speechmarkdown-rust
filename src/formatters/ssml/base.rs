use crate::ast::{AstNode, NodeType};
use crate::error::Result;
use crate::formatters::base::{Formatter, FormatterOptions};

pub type TagAttrs = Vec<(String, String)>;
pub type TagInfo = (String, TagAttrs);

pub fn attrs_insert(attrs: &mut TagAttrs, key: &str, value: String) {
    if let Some(entry) = attrs.iter_mut().find(|(k, _)| k == key) {
        entry.1 = value;
    } else {
        attrs.push((key.to_string(), value));
    }
}

pub fn attrs_merge(existing: &mut TagAttrs, new: TagAttrs) {
    for (k, v) in new {
        attrs_insert(existing, &k, v);
    }
}

pub fn attrs_get<'a>(attrs: &'a TagAttrs, key: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

pub struct SsmlFormatterBase {
    options: FormatterOptions,
    tag_sort_order: Vec<String>,
}

impl SsmlFormatterBase {
    pub fn new(options: FormatterOptions) -> Self {
        let tag_sort_order = Self::create_default_tag_order();

        Self {
            options,
            tag_sort_order,
        }
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
            "amazon:effect".to_string(),
            "amazon:emotion".to_string(),
            "amazon:domain".to_string(),
        ]
    }

    pub fn format_node_internal(&self, node: &AstNode) -> Result<String> {
        self.format_node_with_tags(node)
    }

    fn capitalize_voice_name(name: &str) -> String {
        let valid_names = [
            "Ivy",
            "Joanna",
            "Joey",
            "Justin",
            "Kendra",
            "Kimberly",
            "Matthew",
            "Salli",
            "Brian",
            "Amy",
            "Emma",
            "Geraint",
            "Russell",
            "Nicole",
            "Celine",
            "Mathieu",
            "Dora",
            "Victor",
            "Tatyana",
            "Maxim",
            "Hans",
            "Marlene",
            "Vicki",
            "Aditi",
            "Karl",
            "Giorgio",
            "Carla",
            "Bianca",
            "Lucia",
            "Mizuki",
            "Takumi",
            "Vitoria",
            "Ricardo",
            "Ines",
            "Cristiano",
            "Lea",
            "Zhiyu",
            "Naja",
            "Mads",
            "Gwyneth",
            "Lotte",
            "Ruben",
            "Ewa",
            "Filiz",
            "Penelope",
            "Lupe",
            "Mia",
            "Conchita",
            "Enrique",
            "Miguel",
            "Penny",
            "Astrid",
            "Bjorn",
            "Sofia",
            "Kasper",
            "Seoyeon",
            "Kendra",
            "Salli",
            "Aria",
            "Jenny",
            "Guy",
            "Davis",
            "Amber",
            "Ana",
            "Andrew",
            "Christopher",
            "Eric",
            "Tony",
        ];
        let lower = name.to_lowercase();
        for valid in &valid_names {
            if valid.to_lowercase() == lower {
                return valid.to_string();
            }
        }
        name.to_string()
    }

    fn is_valid_voice_name(name: &str) -> bool {
        let valid_names = [
            "Ivy",
            "Joanna",
            "Joey",
            "Justin",
            "Kendra",
            "Kimberly",
            "Matthew",
            "Salli",
            "Brian",
            "Amy",
            "Emma",
            "Geraint",
            "Russell",
            "Nicole",
            "Celine",
            "Mathieu",
            "Dora",
            "Victor",
            "Tatyana",
            "Maxim",
            "Hans",
            "Marlene",
            "Vicki",
            "Aditi",
            "Karl",
            "Giorgio",
            "Carla",
            "Bianca",
            "Lucia",
            "Mizuki",
            "Takumi",
            "Vitoria",
            "Ricardo",
            "Ines",
            "Cristiano",
            "Lea",
            "Zhiyu",
            "Naja",
            "Mads",
            "Gwyneth",
            "Lotte",
            "Ruben",
            "Ewa",
            "Filiz",
            "Penelope",
            "Lupe",
            "Mia",
            "Conchita",
            "Enrique",
            "Miguel",
            "Penny",
            "Astrid",
            "Bjorn",
            "Sofia",
            "Kasper",
            "Seoyeon",
            "Aria",
            "Jenny",
            "Guy",
            "Davis",
            "Amber",
            "Ana",
            "Andrew",
            "Christopher",
            "Eric",
            "Tony",
        ];
        valid_names.contains(&name)
    }

    pub fn format_node_with_tags(&self, node: &AstNode) -> Result<String> {
        match node.node_type {
            NodeType::Document => self.format_document(node),
            NodeType::Paragraph => self.format_paragraph(node),
            NodeType::SimpleLine => self.format_simple_line(node),
            NodeType::EmptyLine => self.format_empty_line(node),
            NodeType::Section => self.format_section(node),
            NodeType::PlainText => Ok(node.text.clone()),
            NodeType::ShortBreak => self.format_short_break(node),
            NodeType::Break => self.format_break(node),
            NodeType::ShortEmphasisModerate => self.format_emphasis(node, "moderate"),
            NodeType::ShortEmphasisStrong => self.format_emphasis(node, "strong"),
            NodeType::ShortEmphasisNone => self.format_emphasis(node, "none"),
            NodeType::ShortEmphasisReduced => self.format_emphasis(node, "reduced"),
            NodeType::TextModifier => self.format_text_modifier(node),
            NodeType::Audio => self.format_audio(node),
            NodeType::Mark => self.format_mark(node),
            NodeType::ShortIpa => self.format_ipa(node),
            NodeType::BareIpa => self.format_bare_ipa(node),
            NodeType::ShortSub => self.format_short_sub(node),
            NodeType::Expressive => Ok(format!("[{}]", node.text)),
            _ => Ok(node.text.clone()),
        }
    }

    fn format_document(&self, node: &AstNode) -> Result<String> {
        let mut content = String::new();
        let mut children_iter = node.children.iter().peekable();

        while let Some(child) = children_iter.next() {
            if child.node_type == NodeType::Section {
                let mut section_content_raw = String::new();
                while let Some(next_child) = children_iter.peek() {
                    if next_child.node_type == NodeType::Section {
                        break;
                    }
                    let next_child = children_iter.next().unwrap();
                    section_content_raw.push_str(&self.format_node_with_tags(next_child)?);
                }
                let had_leading_newline = section_content_raw.starts_with('\n');
                let section_content = if had_leading_newline {
                    &section_content_raw[1..]
                } else {
                    &section_content_raw
                };

                let section_open = self.format_node_with_tags(child)?;
                let section_close = if !section_open.is_empty() {
                    self.format_section_close(child)?
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
            } else {
                content.push_str(&self.format_node_with_tags(child)?);
            }
        }

        if self.options.include_speak_tag {
            let trimmed = content.trim_end_matches('\n');
            Ok(format!("<speak>\n{}\n</speak>", trimmed))
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
        let mut tags: Vec<TagInfo> = Vec::new();

        if let Some(style) = node.attributes.get("style") {
            if style != "defaults" {
                if let Some(tag_info) = self.attribute_to_tag(style, "") {
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
            if let Some(tag_info) = self.attribute_to_tag(key, value) {
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

        let section_tag_order = ["voice", "lang", "prosody", "emphasis"];
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

    pub fn format_section_close(&self, node: &AstNode) -> Result<String> {
        let mut tags: Vec<TagInfo> = Vec::new();

        if let Some(style) = node.attributes.get("style") {
            if style != "defaults" {
                if let Some(tag_info) = self.attribute_to_tag(style, "") {
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
            if let Some(tag_info) = self.attribute_to_tag(key, value) {
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

        let section_tag_order = ["voice", "lang", "prosody", "emphasis"];
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

    fn format_short_break(&self, node: &AstNode) -> Result<String> {
        let time = node.text.trim_start_matches('[').trim_end_matches(']');
        Ok(format!("<break time=\"{}\"/>", time))
    }

    fn format_break(&self, node: &AstNode) -> Result<String> {
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
        let mut tags: Vec<TagInfo> = Vec::new();
        let mut last_say_as: Option<TagInfo> = None;

        for key in &node.attribute_keys {
            let value = match node.attributes.get(key) {
                Some(v) => v,
                None => continue,
            };
            if let Some(tag_info) = self.attribute_to_tag(key, value) {
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

    pub fn attribute_to_tag(&self, key: &str, value: &str) -> Option<TagInfo> {
        let mut attributes: TagAttrs = Vec::new();

        match key.to_lowercase().as_str() {
            "address" => Some(("say-as".to_string(), {
                vec![("interpret-as".to_string(), "address".to_string())]
            })),
            "date" => Some(("say-as".to_string(), {
                let mut attrs = vec![("interpret-as".to_string(), "date".to_string())];
                if !value.is_empty() {
                    attrs.push(("format".to_string(), value.to_string()));
                }
                attrs
            })),
            "time" => Some(("say-as".to_string(), {
                let mut attrs = Vec::new();
                if !value.is_empty() {
                    attrs.push(("format".to_string(), value.to_string()));
                }
                attrs.push(("interpret-as".to_string(), "time".to_string()));
                attrs
            })),
            "number" | "cardinal" => Some(("say-as".to_string(), {
                vec![("interpret-as".to_string(), "number".to_string())]
            })),
            "ordinal" => Some(("say-as".to_string(), {
                vec![("interpret-as".to_string(), "ordinal".to_string())]
            })),
            "characters" | "chars" | "digits" | "drc" => Some(("say-as".to_string(), {
                vec![("interpret-as".to_string(), "characters".to_string())]
            })),
            "fraction" => Some(("say-as".to_string(), {
                vec![("interpret-as".to_string(), "fraction".to_string())]
            })),
            "unit" => Some(("say-as".to_string(), {
                vec![("interpret-as".to_string(), "unit".to_string())]
            })),
            "interjection" => Some(("say-as".to_string(), {
                vec![("interpret-as".to_string(), "interjection".to_string())]
            })),
            "expletive" | "bleep" => Some(("say-as".to_string(), {
                vec![("interpret-as".to_string(), "expletive".to_string())]
            })),
            "telephone" | "phone" => Some(("say-as".to_string(), {
                vec![("interpret-as".to_string(), "telephone".to_string())]
            })),
            "ipa" => Some(("phoneme".to_string(), {
                let mut attrs = vec![("alphabet".to_string(), "ipa".to_string())];
                if !value.is_empty() {
                    attrs.push(("ph".to_string(), value.to_string()));
                }
                attrs
            })),
            "xsampa" | "praat" | "sil" | "branner" => {
                let translated = translate_to_ipa(&key.to_lowercase(), value)?;
                let mut attrs = vec![("alphabet".to_string(), "ipa".to_string())];
                if !translated.is_empty() {
                    attrs.push(("ph".to_string(), translated));
                }
                Some(("phoneme".to_string(), attrs))
            }
            "sub" => {
                if !value.is_empty() {
                    attributes.push(("alias".to_string(), value.to_string()));
                }
                Some(("sub".to_string(), attributes))
            }
            "voice" => {
                if value.is_empty() || value == "device" {
                    return None;
                }
                let name = Self::capitalize_voice_name(value);
                if !Self::is_valid_voice_name(&name) {
                    return None;
                }
                attributes.push(("name".to_string(), name));
                Some(("voice".to_string(), attributes))
            }
            "lang" => {
                if !value.is_empty() {
                    attributes.push(("xml:lang".to_string(), value.to_string()));
                }
                Some(("lang".to_string(), attributes))
            }
            "rate" => {
                let rate_val = if value.is_empty() { "medium" } else { value };
                attributes.push(("rate".to_string(), rate_val.to_string()));
                Some(("prosody".to_string(), attributes))
            }
            "pitch" => {
                let pitch_val = if value.is_empty() { "medium" } else { value };
                attributes.push(("pitch".to_string(), pitch_val.to_string()));
                Some(("prosody".to_string(), attributes))
            }
            "volume" | "vol" => {
                let vol_val = if value.is_empty() { "medium" } else { value };
                attributes.push(("volume".to_string(), vol_val.to_string()));
                Some(("prosody".to_string(), attributes))
            }
            "timbre" => {
                let timbre_val = if value.is_empty() { "medium" } else { value };
                attributes.push(("pitch".to_string(), timbre_val.to_string()));
                Some(("prosody".to_string(), attributes))
            }
            "emphasis" => {
                let level = if value.is_empty() { "moderate" } else { value };
                attributes.push(("level".to_string(), level.to_string()));
                Some(("emphasis".to_string(), attributes))
            }
            "whisper" => Some(("amazon:effect".to_string(), {
                vec![("name".to_string(), "whispered".to_string())]
            })),
            "excited" => {
                let lower_val = value.to_lowercase();
                if value.is_empty() {
                    Some(("amazon:emotion".to_string(), {
                        vec![
                            ("name".to_string(), "excited".to_string()),
                            ("intensity".to_string(), "medium".to_string()),
                        ]
                    }))
                } else if matches!(lower_val.as_str(), "low" | "medium" | "high") {
                    Some(("amazon:emotion".to_string(), {
                        vec![
                            ("name".to_string(), "excited".to_string()),
                            ("intensity".to_string(), lower_val),
                        ]
                    }))
                } else {
                    None
                }
            }
            "disappointed" => {
                let lower_val = value.to_lowercase();
                if value.is_empty() {
                    Some(("amazon:emotion".to_string(), {
                        vec![
                            ("name".to_string(), "disappointed".to_string()),
                            ("intensity".to_string(), "medium".to_string()),
                        ]
                    }))
                } else if matches!(lower_val.as_str(), "low" | "medium" | "high") {
                    Some(("amazon:emotion".to_string(), {
                        vec![
                            ("name".to_string(), "disappointed".to_string()),
                            ("intensity".to_string(), lower_val),
                        ]
                    }))
                } else {
                    None
                }
            }
            "dj" => Some(("amazon:domain".to_string(), {
                vec![("name".to_string(), "music".to_string())]
            })),
            "newscaster" => Some(("amazon:domain".to_string(), {
                vec![("name".to_string(), "news".to_string())]
            })),
            _ => None,
        }
    }

    pub fn apply_tags_to_text(&self, text: &str, tags: &[TagInfo]) -> Result<String> {
        let mut current_text = text.to_string();

        let mut sorted_tags = tags.to_vec();
        sorted_tags.sort_by_key(|(tag_name, _)| {
            self.tag_sort_order
                .iter()
                .position(|t| t == tag_name)
                .unwrap_or(usize::MAX)
        });

        for (tag_name, attributes) in sorted_tags.iter().rev() {
            let attr_string = format_attr_string_ordered(tag_name, attributes);

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

    pub fn escape_xml(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
}

/// Translate a phonetic-alphabet value to IPA.
///
/// Recognized keys (lowercase): `xsampa`, `praat`, `sil`, `branner`. Returns
/// `None` for any unrecognized key, or for every key when the
/// `phonetic-translation` feature is disabled — callers treat `None` as
/// "drop this modifier."
///
/// Conversion is infallible (garbage in, garbage out) per the upstream crate.
#[cfg(feature = "phonetic-translation")]
pub fn translate_to_ipa(key: &str, value: &str) -> Option<String> {
    if value.is_empty() {
        return Some(String::new());
    }
    match key {
        "xsampa" => Some(ipa_translate::xsampa_to_ipa(value)),
        "praat" => Some(ipa_translate::praat_to_ipa(value)),
        "sil" => Some(ipa_translate::sil_to_ipa(value)),
        "branner" => Some(ipa_translate::branner_to_ipa(value)),
        _ => None,
    }
}

#[cfg(not(feature = "phonetic-translation"))]
pub fn translate_to_ipa(_key: &str, _value: &str) -> Option<String> {
    None
}

pub fn format_attr_string_ordered(tag_name: &str, attributes: &TagAttrs) -> String {
    let fixed_order: Vec<&str> = match tag_name {
        "say-as" => vec!["interpret-as", "format"],
        "phoneme" => vec!["alphabet", "ph"],
        "voice" => vec!["name"],
        "lang" => vec!["xml:lang"],
        "emphasis" => vec!["level"],
        "amazon:effect" => vec!["name"],
        "amazon:emotion" => vec!["name", "intensity"],
        "amazon:domain" => vec!["name"],
        "mstts:express-as" => vec!["style"],
        "sub" => vec!["alias"],
        "prosody" => vec![],
        "google:style" => vec![],
        _ => vec![],
    };

    if fixed_order.is_empty() && !attributes.is_empty() {
        let mut seen = std::collections::HashSet::new();
        let mut parts: Vec<String> = Vec::new();
        for (key, value) in attributes {
            if seen.insert(key.clone()) {
                parts.push(format!("{}=\"{}\"", key, value));
            }
        }
        return parts.join(" ");
    }

    let mut parts: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for key in &fixed_order {
        if let Some(value) = attrs_get(attributes, key) {
            if seen.insert(key.to_string()) {
                parts.push(format!("{}=\"{}\"", key, value));
            }
        }
    }
    for (key, value) in attributes {
        if !fixed_order.contains(&key.as_str()) && seen.insert(key.clone()) {
            parts.push(format!("{}=\"{}\"", key, value));
        }
    }
    parts.join(" ")
}

impl Formatter for SsmlFormatterBase {
    fn format(&self, ast: &AstNode) -> Result<String> {
        self.format_node_with_tags(ast)
    }

    fn format_node(&self, node: &AstNode) -> Result<String> {
        self.format_node_with_tags(node)
    }
}

#[cfg(test)]
mod phonetic_alphabet_tests {
    use super::*;
    use crate::formatters::base::FormatterOptions;

    fn fmt() -> SsmlFormatterBase {
        SsmlFormatterBase::new(FormatterOptions::default())
    }

    #[cfg(feature = "phonetic-translation")]
    #[test]
    fn xsampa_value_becomes_ipa_phoneme_tag() {
        let (tag, attrs) = fmt().attribute_to_tag("xsampa", "spitS").unwrap();
        assert_eq!(tag, "phoneme");
        assert_eq!(attrs_get(&attrs, "alphabet"), Some("ipa"));
        assert_eq!(attrs_get(&attrs, "ph"), Some("spitʃ"));
    }

    #[cfg(feature = "phonetic-translation")]
    #[test]
    fn praat_sil_branner_all_emit_ipa_phoneme() {
        let cases = [
            ("praat", r"p\rta\:ft\^h", "pɹaːtʰ"),
            ("sil", "si=l", "sɪl"),
            ("branner", "br&ae):nE&r^", "bɹæːnɜ˞"),
        ];
        for (key, src, expected_ipa) in cases {
            let (tag, attrs) = fmt().attribute_to_tag(key, src).unwrap();
            assert_eq!(tag, "phoneme", "key {}", key);
            assert_eq!(attrs_get(&attrs, "alphabet"), Some("ipa"), "key {}", key);
            assert_eq!(attrs_get(&attrs, "ph"), Some(expected_ipa), "key {}", key);
        }
    }

    #[cfg(not(feature = "phonetic-translation"))]
    #[test]
    fn phonetic_keys_dropped_when_feature_disabled() {
        for key in ["xsampa", "praat", "sil", "branner"] {
            assert!(
                fmt().attribute_to_tag(key, "anything").is_none(),
                "{} should be dropped without phonetic-translation feature",
                key
            );
        }
    }

    #[cfg(feature = "phonetic-translation")]
    #[test]
    fn empty_value_emits_phoneme_without_ph_attr() {
        let (tag, attrs) = fmt().attribute_to_tag("xsampa", "").unwrap();
        assert_eq!(tag, "phoneme");
        assert_eq!(attrs_get(&attrs, "alphabet"), Some("ipa"));
        assert_eq!(attrs_get(&attrs, "ph"), None);
    }
}
