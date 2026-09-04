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
    #[allow(dead_code)]
    preserve_empty_lines: bool,
}

impl ElevenLabsFormatter {
    pub fn new(_options: FormatterOptions) -> Self {
        Self {
            preserve_empty_lines: true,
        }
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
                let time = Self::break_time_from_text(&node.text);
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
                        ph, node.text
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
                        ph, node.text
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
            // emit markup the model would read aloud.
            NodeType::Audio | NodeType::Mark => {}

            NodeType::Expressive => {
                out.push_str(&format!("[{}]", node.text));
            }

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
    use super::*;
    use crate::formatters::base::Platform;
    use crate::parser::SpeechMarkdownParser;

    fn to_elevenlabs(input: &str) -> String {
        SpeechMarkdownParser::to_ssml(input, Platform::ElevenLabs).unwrap()
    }

    #[test]
    fn short_break_becomes_break_tag() {
        assert_eq!(
            to_elevenlabs("Sample [3s] speech [250ms] markdown"),
            "Sample <break time=\"3s\"/> speech <break time=\"250ms\"/> markdown"
        );
    }

    #[test]
    fn quoted_break_time_becomes_break_tag() {
        assert_eq!(
            to_elevenlabs("Sample [break:\"3s\"] speech [break:'250ms'] markdown"),
            "Sample <break time=\"3s\"/> speech <break time=\"250ms\"/> markdown"
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
    fn expressive_tags_pass_through() {
        assert_eq!(
            to_elevenlabs("He [laugh] and then [applause] left"),
            "He [laugh] and then [applause] left"
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
