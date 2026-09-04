use crate::ast::{AstNode, NodeType};
use crate::error::Result;
use crate::formatters::base::{Formatter, FormatterOptions};

/// Eleven v3 audio-tag dialect for `eleven_v3` / `eleven_v3_conversational`.
///
/// Eleven v3 does not parse SSML — no `<break>`, no `<phoneme>`. Delivery is
/// directed with bracketed natural-language audio tags (`[whispers]`,
/// `[laughs]`, `[pause]`), punctuation (`...`, em-dash) and capitalization.
/// Tags are open-ended prompts interpreted by the model, not an enum: they
/// are best-effort and voice-dependent, and they directionally apply from
/// their insertion point onward (there is no guaranteed "end tag" span).
///
/// Mapping notes:
/// - Breaks lose temporal precision: three steps plus punctuation
///   (`...` / `[short pause]` / `[pause]` / `[long pause]`).
/// - Emphasis avoids mutating the user's words (no CAPS by default).
/// - IPA is emitted in v3's native `"/…/"` slash form.
/// - `#[style]` sections become prefix tags; unknown styles pass through
///   verbatim (v3 treats any bracketed cue as direction).
pub struct ElevenLabsV3Formatter {
    preserve_empty_lines: bool,
}

/// Break strength → v3 pause tag. No temporal precision available.
const BREAK_STRENGTH_TO_TAG: &[(&str, &str)] = &[
    ("none", ""),
    ("x-weak", "..."),
    ("weak", "[short pause]"),
    ("medium", "[pause]"),
    ("strong", "[long pause]"),
    ("x-strong", "[long pause]"),
];

const DEFAULT_PAUSE_TAG: &str = "[pause]";

impl ElevenLabsV3Formatter {
    pub fn new(options: FormatterOptions) -> Self {
        Self {
            preserve_empty_lines: options.preserve_empty_lines,
        }
    }

    /// Parse "2", "0.25", "250ms", "1.5s", … into seconds.
    fn parse_seconds(text: &str) -> Option<f64> {
        let body = text
            .strip_suffix("ms")
            .or_else(|| text.strip_suffix('s'))
            .unwrap_or(text);
        let value: f64 = body.parse().ok()?;
        let scale = if text.ends_with("ms") { 0.001 } else { 1.0 };
        Some(value * scale)
    }

    fn pause_tag_for_seconds(secs: f64) -> &'static str {
        if secs < 0.4 {
            "..."
        } else if secs <= 1.0 {
            "[pause]"
        } else {
            "[long pause]"
        }
    }

    fn strength_to_tag(strength: &str) -> &'static str {
        let normalized = strength.trim().to_lowercase();
        BREAK_STRENGTH_TO_TAG
            .iter()
            .find(|(k, _)| *k == normalized)
            .map(|(_, v)| *v)
            .unwrap_or(DEFAULT_PAUSE_TAG)
    }

    fn break_time_from_text(text: &str) -> &str {
        text.trim_start_matches('[').trim_end_matches(']')
    }

    /// Map a modifier key/value pair to a prefix audio tag (or None to
    /// degrade to plain text). IPA is handled separately by the caller
    /// because it replaces the text instead of prefixing it.
    fn modifier_to_tag(key: &str, value: &str) -> Option<String> {
        let value = value.trim().to_lowercase();
        match key.to_lowercase().as_str() {
            "whisper" => Some("[whispers]".to_string()),
            "excited" => Some("[excited]".to_string()),
            "disappointed" => Some("[disappointed]".to_string()),
            "rate" => match value.as_str() {
                "x-slow" | "slow" => Some("[drawn out]".to_string()),
                "fast" | "x-fast" => Some("[rushed]".to_string()),
                _ => None,
            },
            "volume" | "vol" => match value.as_str() {
                "x-soft" | "soft" | "quiet" => Some("[softly]".to_string()),
                "x-loud" | "loud" => Some("[loudly]".to_string()),
                _ => None,
            },
            "emphasis" => match value.as_str() {
                "strong" => Some("[emphasized]".to_string()),
                "moderate" => Some("[stress on next word]".to_string()),
                "reduced" => Some("[understated]".to_string()),
                _ => None,
            },
            _ => None,
        }
    }

    fn format_node_internal(&self, node: &AstNode, out: &mut String) -> Result<()> {
        match node.node_type {
            NodeType::Document => {
                // Sections prefix the content that follows them (until the
                // next section); there is no closing form on v3.
                let mut iter = node.children.iter().peekable();
                while let Some(child) = iter.next() {
                    if child.node_type == NodeType::Section {
                        let mut section_content = String::new();
                        while let Some(next) = iter.peek() {
                            if next.node_type == NodeType::Section {
                                break;
                            }
                            let next = iter.next().unwrap();
                            self.format_node_internal(next, &mut section_content)?;
                        }
                        out.push_str(&self.section_prefix(child));
                        out.push_str(&section_content);
                    } else {
                        self.format_node_internal(child, out)?;
                    }
                }
            }

            NodeType::PlainText
            | NodeType::PlainTextSpecialChars
            | NodeType::PlainTextEmphasis
            | NodeType::SimpleLine
            | NodeType::Paragraph => {
                out.push_str(&node.text);
                for child in &node.children {
                    self.format_node_internal(child, out)?;
                }
            }

            NodeType::EmptyLine => {
                if self.preserve_empty_lines {
                    out.push('\n');
                }
            }

            NodeType::ShortBreak => {
                let time = Self::break_time_from_text(&node.text);
                let tag = Self::parse_seconds(time)
                    .map(Self::pause_tag_for_seconds)
                    .unwrap_or(DEFAULT_PAUSE_TAG);
                out.push_str(tag);
            }

            NodeType::Break => {
                let strength = node
                    .attributes
                    .get("strength")
                    .unwrap_or(&node.text)
                    .clone();
                out.push_str(Self::strength_to_tag(&strength));
            }

            NodeType::ShortEmphasisStrong => {
                out.push_str("[emphasized] ");
                out.push_str(&node.text);
            }
            NodeType::ShortEmphasisModerate => {
                out.push_str("[stress on next word] ");
                out.push_str(&node.text);
            }
            NodeType::ShortEmphasisReduced => {
                out.push_str("[understated] ");
                out.push_str(&node.text);
            }
            NodeType::ShortEmphasisNone => {
                out.push_str(&node.text);
            }

            NodeType::TextModifier => {
                // IPA replaces the text; other recognized modifiers
                // become prefix tags in declaration order.
                if let Some(ph) = node.attributes.get("ipa").filter(|v| !v.is_empty()) {
                    out.push_str(&format!("\"/{}/\"", ph));
                } else if let Some(alias) = node.attributes.get("sub").filter(|v| !v.is_empty()) {
                    // Substitution: speak the alias instead of the text.
                    out.push_str(alias);
                } else {
                    let mut tags: Vec<String> = Vec::new();
                    for key in &node.attribute_keys {
                        let value = node.attributes.get(key).map(String::as_str).unwrap_or("");
                        if let Some(tag) = Self::modifier_to_tag(key, value) {
                            tags.push(tag);
                        }
                    }
                    for tag in &tags {
                        out.push_str(tag);
                        out.push(' ');
                    }
                    out.push_str(&node.text);
                }
            }

            NodeType::ShortIpa => {
                // v3 native IPA replaces the word with "/phoneme/".
                if let Some(ph) = node.attributes.get("phoneme").filter(|v| !v.is_empty()) {
                    out.push_str(&format!("\"/{}/\"", ph));
                } else {
                    out.push_str(&node.text);
                }
            }

            NodeType::BareIpa => {
                if let Some(ph) = node.attributes.get("ph") {
                    out.push_str(&format!("\"/{}/\"", ph));
                } else {
                    out.push_str(&node.text);
                }
            }

            NodeType::ShortSub => {
                // Speak the alias (the intended spoken form) when present.
                if let Some(alias) = node.attributes.get("alias").filter(|v| !v.is_empty()) {
                    out.push_str(alias);
                } else {
                    out.push_str(&node.text);
                }
            }

            NodeType::Audio | NodeType::Mark => {}

            NodeType::Expressive => {
                out.push_str(&format!("[{}]", node.text));
            }

            NodeType::Section => {
                // Handled by the document walk; standalone formatting of a
                // section still emits its prefix tags.
                out.push_str(&self.section_prefix(node));
            }

            // Modifier node types never appear standalone from the parser.
            _ => {}
        }
        Ok(())
    }

    /// Prefix tags for a `#[…]` section: the bare style passes through as a
    /// natural-language tag; recognized modifier keys map to tempo/volume
    /// tags; everything else is dropped (no v3 equivalent).
    fn section_prefix(&self, node: &AstNode) -> String {
        let mut tags: Vec<String> = Vec::new();

        if let Some(style) = node.attributes.get("style") {
            if style != "defaults" && !style.is_empty() {
                tags.push(format!("[{}]", style));
            }
        }

        for key in &node.attribute_keys {
            if key == "style" {
                continue;
            }
            let value = node.attributes.get(key).map(String::as_str).unwrap_or("");
            if let Some(tag) = Self::modifier_to_tag(key, value) {
                tags.push(tag);
            }
        }

        if tags.is_empty() {
            String::new()
        } else {
            format!("{} ", tags.join(" "))
        }
    }
}

impl Formatter for ElevenLabsV3Formatter {
    fn format(&self, ast: &AstNode) -> Result<String> {
        let mut out = String::new();
        self.format_node_internal(ast, &mut out)?;
        Ok(out)
    }

    fn format_node(&self, node: &AstNode) -> Result<String> {
        let mut out = String::new();
        self.format_node_internal(node, &mut out)?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use crate::formatters::base::Platform;
    use crate::parser::SpeechMarkdownParser;

    fn to_v3(input: &str) -> String {
        SpeechMarkdownParser::to_ssml(input, Platform::ElevenLabsV3).unwrap()
    }

    #[test]
    fn short_breaks_map_to_pause_steps() {
        assert_eq!(to_v3("Sample [250ms] speech"), "Sample ... speech");
        assert_eq!(to_v3("Sample [0.5s] speech"), "Sample [pause] speech");
        assert_eq!(to_v3("Sample [2s] speech"), "Sample [long pause] speech");
    }

    #[test]
    fn break_strengths_map_to_pause_tags() {
        assert_eq!(to_v3("[break:\"none\"]"), "");
        assert_eq!(to_v3("[break:\"x-weak\"]"), "...");
        assert_eq!(to_v3("[break:\"weak\"]"), "[short pause]");
        assert_eq!(to_v3("[break:\"medium\"]"), "[pause]");
        assert_eq!(to_v3("[break:\"strong\"]"), "[long pause]");
        assert_eq!(to_v3("[break:\"x-strong\"]"), "[long pause]");
        assert_eq!(to_v3("[break:\"bogus\"]"), "[pause]");
    }

    #[test]
    fn emphasis_maps_to_tags_not_caps() {
        assert_eq!(to_v3("very ++important++"), "very [emphasized] important");
        assert_eq!(
            to_v3("a +little+ bit"),
            "a [stress on next word] little bit"
        );
        assert_eq!(to_v3("a -little- bit"), "a [understated] little bit");
        assert_eq!(to_v3("~whatever~"), "whatever");
    }

    #[test]
    fn whisper_and_emotion_modifiers_prefix_tags() {
        assert_eq!(
            to_v3("(it's a secret)[whisper]"),
            "[whispers] it's a secret"
        );
        assert_eq!(to_v3("(great news)[excited]"), "[excited] great news");
    }

    #[test]
    fn rate_and_volume_map_to_tempo_tags() {
        assert_eq!(
            to_v3("(read this)[rate:\"fast\";volume:\"loud\"]"),
            "[rushed] [loudly] read this"
        );
        assert_eq!(to_v3("(slowly)[rate:\"slow\"]"), "[drawn out] slowly");
    }

    #[test]
    fn unsupported_modifiers_degrade_to_text() {
        assert_eq!(to_v3("(hello)[voice:\"Brian\"]"), "hello");
        assert_eq!(to_v3("(bonjour)[lang:\"fr-FR\"]"), "bonjour");
        assert_eq!(to_v3("(42)[number]"), "42");
    }

    #[test]
    fn ipa_becomes_native_slash_form() {
        assert_eq!(to_v3("(speech)/spitʃ/"), "\"/spitʃ/\"");
        assert_eq!(to_v3("(word)[ipa:\"wɜːd\"]"), "\"/wɜːd/\"");
    }

    #[test]
    fn sub_speaks_alias() {
        assert_eq!(to_v3("{AL}aluminum"), "aluminum");
    }

    #[test]
    fn sections_become_prefix_tags() {
        assert_eq!(to_v3("#[excited] Hello world"), "[excited]  Hello world");
        // Unknown styles pass through as natural-language direction.
        assert_eq!(to_v3("#[sarcastic] nice"), "[sarcastic]  nice");
        // Recognized section modifiers map like inline ones.
        assert_eq!(to_v3("#[rate:\"slow\"] steady"), "[drawn out]  steady");
        // defaults produces nothing.
        assert_eq!(to_v3("#[defaults] plain"), " plain");
    }

    #[test]
    fn expressive_tags_pass_through() {
        assert_eq!(
            to_v3("He [laugh] and then [applause] left"),
            "He [laugh] and then [applause] left"
        );
    }

    #[test]
    fn audio_and_mark_are_dropped() {
        // Dropped nodes leave the surrounding spacing in place.
        assert_eq!(
            to_v3("Hello [mark:chapter1] ![sfx](\"https://x/y.mp3\") world"),
            "Hello   world"
        );
    }

    #[test]
    fn no_speak_wrapper_and_no_escaping() {
        assert_eq!(to_v3("1 < 2 & 3 > 0"), "1 < 2 & 3 > 0");
    }
}
