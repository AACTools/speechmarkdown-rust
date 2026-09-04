use crate::ast::{AstNode, NodeType};
use crate::error::Result;
use crate::formatters::base::{Formatter, FormatterOptions};

/// Break strength → approximate duration, matching the speechmarkdown-js
/// ElevenLabsFormatter reference. ElevenLabs only accepts `time` (no
/// `strength` attribute), up to 3 seconds.
const BREAK_STRENGTH_TO_DURATION: &[(&str, &str)] = &[
    ("none", "0s"),
    ("x-weak", "0.2s"),
    ("weak", "0.35s"),
    ("medium", "0.5s"),
    ("strong", "0.8s"),
    ("x-strong", "1.2s"),
];

const DEFAULT_BREAK_DURATION: &str = "0.5s";

/// ElevenLabs pre-v3 models accept pauses up to 3 seconds; longer
/// requested breaks are clamped (values beyond that are rejected or
/// destabilize the generation).
const MAX_BREAK_SECONDS: f64 = 3.0;

/// ElevenLabs prompt markup for the pre-v3 model family
/// (`eleven_multilingual_v2`, `eleven_flash_v2_5`, `eleven_flash_v2`,
/// `eleven_turbo_v2`).
///
/// These models do not parse SSML documents, but they do understand two
/// inline XML-style tags:
///
/// - `<break time="x.xs" />` — exact pause, up to 3 seconds, seconds format
/// - `<phoneme alphabet="ipa" ph="…" />` — pronunciation control,
///   `eleven_flash_v2` / `eleven_turbo_v2` only, English only
///
/// Everything else in SpeechMarkdown degrades to plain text. Output is a
/// bare prompt: no `<speak>` wrapper and no XML escaping (matching the
/// speechmarkdown-js reference formatter and the shared test corpus).
///
/// `eleven_v3` models must use the audio-tag dialect instead
/// (`Platform::ElevenLabsV3`): they do not parse `<break>` at all.
pub struct ElevenLabsFormatter {
    preserve_empty_lines: bool,
}

impl ElevenLabsFormatter {
    pub fn new(options: FormatterOptions) -> Self {
        Self {
            preserve_empty_lines: options.preserve_empty_lines,
        }
    }

    /// Parse a duration like "2", "0.25", "250ms", "1.5s" into seconds.
    fn parse_seconds(text: &str) -> Option<f64> {
        let body = text
            .strip_suffix("ms")
            .or_else(|| text.strip_suffix('s'))
            .unwrap_or(text);
        let value: f64 = body.parse().ok()?;
        let scale = if text.ends_with("ms") { 0.001 } else { 1.0 };
        Some(value * scale)
    }

    /// Normalize a break duration to ElevenLabs' documented seconds
    /// format ("Break time should be described in seconds"), clamped to
    /// the 3s limit: "250ms" → "0.25s", "10s" → "3s", "1.5s" unchanged.
    fn normalize_break_time(time: &str) -> String {
        match Self::parse_seconds(time) {
            Some(secs) => Self::format_seconds(secs.min(MAX_BREAK_SECONDS)),
            None => time.to_string(),
        }
    }

    /// Format seconds without trailing zeros ("2s", "0.25s", "1.2s").
    /// Three decimals cover millisecond precision.
    fn format_seconds(secs: f64) -> String {
        let mut s = format!("{secs:.3}");
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
        format!("{s}s")
    }

    fn map_strength_to_time(strength: &str) -> String {
        let normalized = strength.trim().to_lowercase();
        BREAK_STRENGTH_TO_DURATION
            .iter()
            .find(|(k, _)| *k == normalized)
            .map(|(_, v)| (*v).to_string())
            .unwrap_or_else(|| DEFAULT_BREAK_DURATION.to_string())
    }

    /// Strip the `[` / `]` the parser keeps in `ShortBreak::text`
    /// (e.g. "[2s]" → "2s").
    fn break_time_from_text(text: &str) -> &str {
        text.trim_start_matches('[').trim_end_matches(']')
    }

    /// Escape a double-quoted attribute value. The prompt body stays
    /// unescaped (ElevenLabs is not an XML document), but attribute
    /// quotes would break the tag itself.
    fn escape_attr(value: &str) -> String {
        value.replace('"', "&quot;")
    }

    fn format_node_internal(&self, node: &AstNode, out: &mut String) -> Result<()> {
        match node.node_type {
            NodeType::Document => {
                for child in &node.children {
                    self.format_node_internal(child, out)?;
                }
            }

            // Sections have no pre-v3 equivalent: the marker itself is
            // dropped and the section's content flows as plain text.
            NodeType::Section => {}

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
                let time = Self::normalize_break_time(Self::break_time_from_text(&node.text));
                out.push_str(&format!("<break time=\"{}\"/>", time));
            }

            NodeType::Break => {
                let strength = node
                    .attributes
                    .get("strength")
                    .unwrap_or(&node.text)
                    .clone();
                let time = Self::map_strength_to_time(&strength);
                out.push_str(&format!("<break time=\"{}\"/>", time));
            }

            NodeType::ShortEmphasisModerate
            | NodeType::ShortEmphasisStrong
            | NodeType::ShortEmphasisNone
            | NodeType::ShortEmphasisReduced => {
                out.push_str(&node.text);
            }

            NodeType::TextModifier => {
                let phoneme = node.attributes.get("ipa");
                if let Some(ph) = phoneme.filter(|ph| !ph.is_empty()) {
                    out.push_str(&format!(
                        "<phoneme alphabet=\"ipa\" ph=\"{}\">{}</phoneme>",
                        Self::escape_attr(ph),
                        node.text
                    ));
                } else {
                    out.push_str(&node.text);
                }
            }

            NodeType::ShortIpa => {
                let phoneme = node.attributes.get("phoneme");
                if let Some(ph) = phoneme.filter(|ph| !ph.is_empty()) {
                    out.push_str(&format!(
                        "<phoneme alphabet=\"ipa\" ph=\"{}\">{}</phoneme>",
                        Self::escape_attr(ph),
                        node.text
                    ));
                } else {
                    out.push_str(&node.text);
                }
            }

            NodeType::BareIpa => {
                // A bare `/ipa/` has no display word; keep the phoneme
                // characters as text (reference-formatter behavior).
                if let Some(ph) = node.attributes.get("ph") {
                    out.push_str(ph);
                } else {
                    out.push_str(&node.text);
                }
            }

            NodeType::ShortSub => {
                out.push_str(&node.text);
            }

            // No pre-v3 equivalent: drop rather than speak a URL or
            // emit markup the model would read aloud. Expressive audio
            // tags ([laugh], …) are v3-exclusive — pre-v3 models read
            // them aloud as literal text — so they are dropped here and
            // emitted only by the elevenlabs-v3 dialect.
            NodeType::Audio | NodeType::Mark | NodeType::Expressive => {}

            // Modifier node types never appear standalone from the parser.
            _ => {}
        }
        Ok(())
    }
}

impl Formatter for ElevenLabsFormatter {
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

    fn to_elevenlabs(input: &str) -> String {
        SpeechMarkdownParser::to_ssml(input, Platform::ElevenLabs).unwrap()
    }

    #[test]
    fn short_break_becomes_break_tag() {
        assert_eq!(
            to_elevenlabs("Sample [3s] speech [250ms] markdown"),
            "Sample <break time=\"3s\"/> speech <break time=\"0.25s\"/> markdown"
        );
    }

    #[test]
    fn quoted_break_time_becomes_break_tag() {
        assert_eq!(
            to_elevenlabs("Sample [break:\"3s\"] speech [break:'250ms'] markdown"),
            "Sample <break time=\"3s\"/> speech <break time=\"0.25s\"/> markdown"
        );
    }

    #[test]
    fn break_strength_maps_to_duration() {
        assert_eq!(
            to_elevenlabs("[break:\"medium\"]"),
            "<break time=\"0.5s\"/>"
        );
        assert_eq!(
            to_elevenlabs("[break:\"x-strong\"]"),
            "<break time=\"1.2s\"/>"
        );
        assert_eq!(to_elevenlabs("[break:\"none\"]"), "<break time=\"0s\"/>");
        // Unknown strength falls back to medium duration.
        assert_eq!(to_elevenlabs("[break:\"bogus\"]"), "<break time=\"0.5s\"/>");
    }

    #[test]
    fn breaks_clamp_and_normalize_to_seconds() {
        // Pre-v3 models accept at most 3s and the documented format is
        // seconds ("Break time should be described in seconds").
        assert_eq!(
            to_elevenlabs("Wait [10s] now"),
            "Wait <break time=\"3s\"/> now"
        );
        assert_eq!(
            to_elevenlabs("Wait [3500ms] now"),
            "Wait <break time=\"3s\"/> now"
        );
        assert_eq!(
            to_elevenlabs("Wait [250ms] now"),
            "Wait <break time=\"0.25s\"/> now"
        );
        assert_eq!(
            to_elevenlabs("Wait [1.50s] now"),
            "Wait <break time=\"1.5s\"/> now"
        );
        assert_eq!(
            to_elevenlabs("Wait [2s] now"),
            "Wait <break time=\"2s\"/> now"
        );
    }

    #[test]
    fn malformed_break_numbers_are_not_breaks() {
        // Not valid durations: plain text passthrough (matches the
        // speechmarkdown-js grammar, which only accepts \d+(\.\d+)?(s|ms)).
        for word in [
            "1.2.3s", "1..5s", "apps", "infs", "s", "5.s", ".5s", "0.s", "-2s", "+2s", "1e3s",
        ] {
            let out = to_elevenlabs(&format!("x [{word}] y"));
            assert_eq!(out, format!("x [{word}] y"), "word {word}");
        }
    }

    #[test]
    fn phoneme_attribute_quotes_are_escaped() {
        assert_eq!(
            to_elevenlabs("(x)[ipa:\"a\"b\"]"),
            "<phoneme alphabet=\"ipa\" ph=\"a&quot;b\">x</phoneme>"
        );
    }

    #[test]
    fn no_speak_wrapper_and_no_escaping() {
        assert_eq!(
            to_elevenlabs("1 < 2 & 3 > 0 \"yes\" 'no'"),
            "1 < 2 & 3 > 0 \"yes\" 'no'"
        );
    }

    #[test]
    fn empty_modifier_list_strips_to_text() {
        assert_eq!(to_elevenlabs("Some (text)[]"), "Some text");
    }

    #[test]
    fn unsupported_modifiers_degrade_to_text() {
        assert_eq!(
            to_elevenlabs("(read this)[rate:\"fast\";volume:\"loud\"]"),
            "read this"
        );
        assert_eq!(to_elevenlabs("++important++"), "important");
        assert_eq!(to_elevenlabs("(hello)[voice:\"Brian\"]"), "hello");
    }

    #[test]
    fn ipa_modifier_emits_phoneme_tag() {
        assert_eq!(
            to_elevenlabs("(piccolo)[ipa:\"pɪkəloʊ\"]"),
            "<phoneme alphabet=\"ipa\" ph=\"pɪkəloʊ\">piccolo</phoneme>"
        );
    }

    #[test]
    fn short_ipa_emits_phoneme_tag() {
        assert_eq!(
            to_elevenlabs("(speech)/spitʃ/"),
            "<phoneme alphabet=\"ipa\" ph=\"spitʃ\">speech</phoneme>"
        );
    }

    #[test]
    fn expressive_tags_are_dropped() {
        // Audio tags are eleven_v3-exclusive; pre-v3 models read them
        // aloud as literal text, so the pre-v3 dialect drops them.
        assert_eq!(
            to_elevenlabs("He [laugh] and then [applause] left"),
            "He  and then  left"
        );
    }

    #[test]
    fn sections_are_dropped_content_flows() {
        assert_eq!(to_elevenlabs("#[excited] Hello world"), " Hello world");
    }

    #[test]
    fn sub_keeps_display_text() {
        assert_eq!(to_elevenlabs("{AL}aluminum"), "AL");
    }

    #[test]
    fn audio_and_mark_are_dropped() {
        // Dropped nodes leave the surrounding spacing in place.
        assert_eq!(
            to_elevenlabs("Hello [mark:chapter1] ![sfx](\"https://x/y.mp3\") world"),
            "Hello   world"
        );
    }
}
