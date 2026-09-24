use crate::ast::{AstNode, NodeType};
use crate::error::Result;
use crate::formatters::base::{Formatter, FormatterOptions};

/// Gemini 3.8 TTS dialect (`gemini-3.8-flash-tts` / `gemini-3.8-flash-lite-tts`).
///
/// Gemini TTS does not parse SSML — the transcript is a verbatim script the
/// model performs, directed with a small inline vocabulary:
///
/// - Angle-bracket vocal bursts: `<laugh>`, `<sigh>`, `<gasp>`, `<cough>`, …
/// - Angle-bracket pauses: `<short pause>`, `<long pause>` (two steps only;
///   finer beats use punctuation such as `...`)
/// - CAPITALIZATION for emphasis
/// - Plain-text disfluencies ("mhm", "oh", "yeah") read naturally — the
///   prompting guide asks for transcripts "written like real speech"
///
/// Sustained delivery (emotion, pace, pitch, volume) belongs in the
/// turn-level `speech_metadata.style` field, which is an engine/channel
/// concern, not expressible inline. Inline modifiers without a Gemini
/// equivalent (rate, pitch, volume, excited, disappointed, generic styles)
/// are therefore dropped; see `capabilities::gemini_capabilities`.
///
/// Mapping notes:
/// - Emphasis collapses to one mechanism: both `+moderate+` and
///   `++strong++` become CAPS (Gemini has no emphasis-level control).
/// - Sound effects (`[applause]`, `[boo]`) are dropped: the prompting
///   guide says to avoid non-vocal sound-effect tags.
/// - Interjection tags (`[mhm]`, `[oh]`, `[wow]`, …) become plain text so
///   the model speaks them as natural conversational backchannels.
/// - IPA has no equivalent (no `<phoneme>`); the word itself is spoken —
///   Gemini 3.8 is trained for difficult pronunciations.
/// - Only English angle-bracket tags are recommended by Google, including
///   for non-English transcripts; this dialect emits them unchanged.
pub struct GeminiFormatter {
    preserve_empty_lines: bool,
}

/// What happens to a parsed `[tag]` expressive node.
enum ExpressiveMapping {
    /// Emit as an angle-bracket vocal burst, normalized to the documented
    /// Gemini form where one exists.
    Angle(&'static str),
    /// Emit as plain text: a conversational disfluency the model speaks.
    PlainText,
    /// Drop entirely (sound effects).
    Drop,
}

/// Map SpeechMarkdown expressive tags → Gemini inline vocabulary.
///
/// Policy, in three tiers:
/// - Tags with a documented Gemini angle-bracket form map to it, with
///   alias normalization ([cheering] → <cheer>, [whew] → <phew>,
///   [ahem] → <throat-clearing>).
/// - Other recognized human vocalizations ([wheeze], [sniff], [hiccup],
///   [hum], [shush]) also become angle tags: Gemini's list is
///   "recommended, not exhaustive" and an angle-bracket direction is
///   safer than inserting the bare word ("wheeze"), which the TTS would
///   pronounce literally.
/// - Everything else (interjections like [mhm]/[wow], unknown tags like
///   [squee]) becomes plain text or is dropped — an arbitrary bracketed
///   cue risks being read literally, and the word itself is the honest
///   degradation for backchannels.
fn expressive_mapping(tag: &str) -> ExpressiveMapping {
    match tag {
        // Documented vocal bursts (with alias normalization to the
        // documented spelling).
        "laugh" => ExpressiveMapping::Angle("laugh"),
        "laughter" => ExpressiveMapping::Angle("laughter"),
        "sigh" => ExpressiveMapping::Angle("sigh"),
        "cough" => ExpressiveMapping::Angle("cough"),
        "cheer" | "cheering" => ExpressiveMapping::Angle("cheer"),
        "cry" | "crying" => ExpressiveMapping::Angle("cry"),
        "gasp" => ExpressiveMapping::Angle("gasp"),
        "groan" | "groaning" => ExpressiveMapping::Angle("groan"),
        "giggle" => ExpressiveMapping::Angle("giggle"),
        "moan" => ExpressiveMapping::Angle("moan"),
        "pant" => ExpressiveMapping::Angle("pant"),
        "scream" => ExpressiveMapping::Angle("scream"),
        "sneeze" => ExpressiveMapping::Angle("sneeze"),
        "whimper" => ExpressiveMapping::Angle("whimper"),
        "yawn" => ExpressiveMapping::Angle("yawn"),
        "tsk" => ExpressiveMapping::Angle("tsk"),
        "throat-clear" | "ahem" => ExpressiveMapping::Angle("throat-clearing"),
        "whew" => ExpressiveMapping::Angle("phew"),

        // Human vocalizations off the recommended list — still vocal, so
        // they pass through in angle brackets.
        "wheeze" => ExpressiveMapping::Angle("wheeze"),
        "sniff" => ExpressiveMapping::Angle("sniff"),
        "hiccup" => ExpressiveMapping::Angle("hiccup"),
        "hum" => ExpressiveMapping::Angle("hum"),
        "shush" => ExpressiveMapping::Angle("shush"),
        "pfft" => ExpressiveMapping::Angle("pff"),
        "phew" => ExpressiveMapping::Angle("phew"),
        "tsk-tsk" => ExpressiveMapping::Angle("tsk"),

        // Documented Gemini bursts added to the parser vocabulary —
        // emit their documented angle-bracket spelling.
        "chuckle" | "chuckles" => ExpressiveMapping::Angle("chuckle"),
        "snicker" => ExpressiveMapping::Angle("snicker"),
        "snort" => ExpressiveMapping::Angle("snort"),
        "sob" => ExpressiveMapping::Angle("sob"),
        "shriek" => ExpressiveMapping::Angle("shriek"),
        "shout" => ExpressiveMapping::Angle("shout"),
        "growl" => ExpressiveMapping::Angle("growl"),
        "grunt" => ExpressiveMapping::Angle("grunt"),
        "hiss" => ExpressiveMapping::Angle("hiss"),
        "grr" => ExpressiveMapping::Angle("grr"),
        "breath" => ExpressiveMapping::Angle("breath"),
        "exhales" => ExpressiveMapping::Angle("exhales"),
        "argh" => ExpressiveMapping::Angle("argh"),

        // Conversational backchannels/disfluencies: spoken, not tagged.
        "hmm" | "mhm" | "mm-hmm" | "uh-huh" | "yeah" | "huh" | "oh" | "mmm" | "ooh" | "meh"
        | "eek" | "bleh" | "wow" | "yay" | "hurray" | "psst" | "shh" | "uh-oh" | "umph" => {
            ExpressiveMapping::PlainText
        }

        // Sound effects: the prompting guide says avoid these.
        "applause" | "boo" => ExpressiveMapping::Drop,

        // Any other bracketed cue is spoken as plain text: Gemini's tag
        // vocabulary is directed at vocal bursts, and an unrecognized
        // tag risks being read literally — plain text is the safe
        // degradation.
        _ => ExpressiveMapping::PlainText,
    }
}

/// Break strength → Gemini pause. Two tag steps plus ellipsis.
const BREAK_STRENGTH_TO_PAUSE: &[(&str, &str)] = &[
    ("none", ""),
    ("x-weak", "..."),
    ("weak", "<short pause>"),
    ("medium", "<short pause>"),
    ("strong", "<long pause>"),
    ("x-strong", "<long pause>"),
];

const DEFAULT_PAUSE: &str = "<short pause>";

impl GeminiFormatter {
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

    /// Duration → Gemini pause. Sub-beat pauses become ellipses (the
    /// prompting guide's "natural hesitation" mechanism); the two pause
    /// tags cover the rest. No millisecond precision exists.
    fn pause_for_seconds(secs: f64) -> &'static str {
        if secs < 0.35 {
            "..."
        } else if secs < 1.2 {
            "<short pause>"
        } else {
            "<long pause>"
        }
    }

    fn strength_to_pause(strength: &str) -> &str {
        let normalized = strength.trim().to_lowercase();
        BREAK_STRENGTH_TO_PAUSE
            .iter()
            .find(|(k, _)| *k == normalized)
            .map(|(_, v)| *v)
            .unwrap_or(DEFAULT_PAUSE)
    }

    fn break_time_from_text(text: &str) -> &str {
        text.trim_start_matches('[').trim_end_matches(']')
    }

    /// Inline modifier key/value → optional `<whispers>` prefix tag.
    /// Whisper is the only SpeechMarkdown modifier with an inline Gemini
    /// equivalent; everything else is either CAPS-mapped (emphasis) or a
    /// turn-level concern handled by the engine's style channel.
    fn modifier_to_tag(key: &str) -> Option<&'static str> {
        match key.to_lowercase().as_str() {
            "whisper" => Some("<whispers>"),
            _ => None,
        }
    }

    /// Does this modifier key force CAPS on the modified text? Only
    /// moderate/strong do (CAPS is Gemini's only emphasis mechanism);
    /// reduced emphasis has no Gemini form and passes through unchanged.
    fn modifier_caps_text(key: &str, value: &str) -> bool {
        key.eq_ignore_ascii_case("emphasis")
            && matches!(
                value.trim().to_lowercase().as_str(),
                "strong" | "moderate" | ""
            )
    }

    fn format_node_internal(&self, node: &AstNode, out: &mut String) -> Result<()> {
        match node.node_type {
            NodeType::Document => {
                // Sections prefix the content that follows them (until the
                // next section); there is no closing form.
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
                let pause = Self::parse_seconds(time)
                    .map(Self::pause_for_seconds)
                    .unwrap_or(DEFAULT_PAUSE);
                out.push_str(pause);
            }

            NodeType::Break => {
                let strength = node
                    .attributes
                    .get("strength")
                    .unwrap_or(&node.text)
                    .clone();
                out.push_str(Self::strength_to_pause(&strength));
            }

            NodeType::ShortEmphasisStrong | NodeType::ShortEmphasisModerate => {
                // CAPS is Gemini's documented emphasis mechanism; there is
                // no level control, so moderate and strong collapse.
                out.push_str(&node.text.to_uppercase());
            }
            NodeType::ShortEmphasisReduced | NodeType::ShortEmphasisNone => {
                out.push_str(&node.text);
            }

            NodeType::TextModifier => {
                // Substitution speaks the alias; IPA and say-as have no
                // inline equivalent (text normalization is native), so the
                // word itself is spoken; whisper gets a prefix tag;
                // emphasis gets CAPS; the rest is dropped.
                if let Some(alias) = node.attributes.get("sub").filter(|v| !v.is_empty()) {
                    out.push_str(alias);
                } else {
                    let mut prefix = String::new();
                    let mut caps = false;
                    for key in &node.attribute_keys {
                        if Self::modifier_caps_text(
                            key,
                            node.attributes.get(key).map(String::as_str).unwrap_or(""),
                        ) {
                            caps = true;
                        } else if let Some(tag) = Self::modifier_to_tag(key) {
                            prefix.push_str(tag);
                            prefix.push(' ');
                        }
                    }
                    out.push_str(&prefix);
                    if caps {
                        out.push_str(&node.text.to_uppercase());
                    } else {
                        out.push_str(&node.text);
                    }
                }
            }

            NodeType::ShortIpa | NodeType::BareIpa => {
                // No phoneme mechanism; speak the word itself.
                out.push_str(&node.text);
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

            NodeType::Expressive => match expressive_mapping(&node.text) {
                ExpressiveMapping::Angle(tag) => out.push_str(&format!("<{tag}>")),
                ExpressiveMapping::PlainText => out.push_str(&node.text),
                ExpressiveMapping::Drop => {}
            },

            NodeType::Section => {
                // Handled by the document walk; standalone formatting of a
                // section still emits its prefix tag.
                out.push_str(&self.section_prefix(node));
            }

            // Modifier node types never appear standalone from the parser.
            _ => {}
        }
        Ok(())
    }

    /// Prefix for a `#[…]` section. Whisper maps to `<whispers>`; style,
    /// rate, pitch, volume and the rest have no inline Gemini form — they
    /// belong in the engine's `speech_metadata.style` channel — so they
    /// produce nothing here.
    fn section_prefix(&self, node: &AstNode) -> String {
        // `#[whisper]` and `#[style:whisper]` are two spellings of the
        // same request — emit at most one prefix tag.
        let style_whisper = node
            .attributes
            .get("style")
            .is_some_and(|s| matches!(s.as_str(), "whisper" | "whispering"));
        let key_whisper = !style_whisper
            && node
                .attribute_keys
                .iter()
                .any(|k| Self::modifier_to_tag(k).is_some());

        if style_whisper || key_whisper {
            "<whispers> ".to_string()
        } else {
            String::new()
        }
    }
}

impl Formatter for GeminiFormatter {
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

    fn to_gemini(input: &str) -> String {
        SpeechMarkdownParser::to_ssml(input, Platform::Gemini).unwrap()
    }

    #[test]
    fn short_breaks_map_to_pause_steps() {
        assert_eq!(to_gemini("Sample [250ms] speech"), "Sample ... speech");
        assert_eq!(
            to_gemini("Sample [0.5s] speech"),
            "Sample <short pause> speech"
        );
        assert_eq!(
            to_gemini("Sample [2s] speech"),
            "Sample <long pause> speech"
        );
    }

    #[test]
    fn break_strengths_map_to_pauses() {
        assert_eq!(to_gemini("[break:\"none\"]"), "");
        assert_eq!(to_gemini("[break:\"x-weak\"]"), "...");
        assert_eq!(to_gemini("[break:\"weak\"]"), "<short pause>");
        assert_eq!(to_gemini("[break:\"medium\"]"), "<short pause>");
        assert_eq!(to_gemini("[break:\"strong\"]"), "<long pause>");
        assert_eq!(to_gemini("[break:\"x-strong\"]"), "<long pause>");
        assert_eq!(to_gemini("[break:\"bogus\"]"), "<short pause>");
    }

    #[test]
    fn emphasis_maps_to_caps() {
        assert_eq!(to_gemini("very ++important++"), "very IMPORTANT");
        assert_eq!(to_gemini("a +little+ bit"), "a LITTLE bit");
        assert_eq!(to_gemini("a -little- bit"), "a little bit");
        assert_eq!(to_gemini("~whatever~"), "whatever");
    }

    #[test]
    fn whisper_gets_prefix_tag() {
        assert_eq!(
            to_gemini("(it's a secret)[whisper]"),
            "<whispers> it's a secret"
        );
    }

    #[test]
    fn inline_emphasis_modifier_caps() {
        assert_eq!(to_gemini("(wow)[emphasis:\"strong\"]"), "WOW");
        // Reduced emphasis has no Gemini form — never CAPS.
        assert_eq!(to_gemini("(nice)[emphasis:\"reduced\"]"), "nice");
    }

    #[test]
    fn prosody_modifiers_drop_inline() {
        // Turn-level delivery belongs in speech_metadata.style (engine
        // channel), not the transcript.
        assert_eq!(
            to_gemini("(read this)[rate:\"fast\";volume:\"loud\"]"),
            "read this"
        );
        assert_eq!(to_gemini("(slowly)[rate:\"slow\"]"), "slowly");
        assert_eq!(to_gemini("(great news)[excited]"), "great news");
        assert_eq!(to_gemini("(oh well)[disappointed]"), "oh well");
    }

    #[test]
    fn unsupported_modifiers_degrade_to_text() {
        assert_eq!(to_gemini("(hello)[voice:\"Brian\"]"), "hello");
        assert_eq!(to_gemini("(bonjour)[lang:\"fr-FR\"]"), "bonjour");
        assert_eq!(to_gemini("(42)[number]"), "42");
    }

    #[test]
    fn ipa_speaks_the_word() {
        // No <phoneme> equivalent; Gemini handles pronunciation natively.
        assert_eq!(to_gemini("(speech)/spitʃ/"), "speech");
        assert_eq!(to_gemini("(word)[ipa:\"wɜːd\"]"), "word");
    }

    #[test]
    fn sub_speaks_alias() {
        assert_eq!(to_gemini("{AL}aluminum"), "aluminum");
    }

    #[test]
    fn vocal_bursts_become_angle_tags() {
        assert_eq!(
            to_gemini("He [laugh] and then [gasp] left"),
            "He <laugh> and then <gasp> left"
        );
        assert_eq!(to_gemini("Well [sigh] fine"), "Well <sigh> fine");
        // Aliases normalize to the documented spelling.
        assert_eq!(to_gemini("[cheering]"), "<cheer>");
        assert_eq!(to_gemini("[throat-clear]"), "<throat-clearing>");
        assert_eq!(to_gemini("[whew]"), "<phew>");
        // Documented bursts added to the parser vocabulary.
        assert_eq!(to_gemini("[chuckle]"), "<chuckle>");
        assert_eq!(to_gemini("[chuckles]"), "<chuckle>");
        assert_eq!(to_gemini("[snort]"), "<snort>");
        assert_eq!(to_gemini("[breath]"), "<breath>");
    }

    #[test]
    fn backchannels_become_plain_text() {
        assert_eq!(to_gemini("I [mhm] agree"), "I mhm agree");
        assert_eq!(to_gemini("[oh] really"), "oh really");
    }

    #[test]
    fn sound_effects_are_dropped() {
        // The prompting guide says avoid non-vocal sound effects.
        assert_eq!(
            to_gemini("He [laugh] and then [applause] left"),
            "He <laugh> and then  left"
        );
        assert_eq!(to_gemini("[boo] the villain"), " the villain");
    }

    #[test]
    fn sections_map_to_whisper_tag_only() {
        assert_eq!(
            to_gemini("#[whisper] secret stuff"),
            "<whispers>  secret stuff"
        );
        // The style spelling maps identically.
        assert_eq!(
            to_gemini("#[style:whisper] secret stuff"),
            "<whispers>  secret stuff"
        );
        // Styles are an engine-channel concern (speech_metadata.style).
        assert_eq!(to_gemini("#[excited] Hello world"), " Hello world");
        assert_eq!(to_gemini("#[sarcastic] nice"), " nice");
        assert_eq!(to_gemini("#[defaults] plain"), " plain");
    }

    #[test]
    fn audio_and_mark_are_dropped() {
        assert_eq!(
            to_gemini("Hello [mark:chapter1] ![sfx](\"https://x/y.mp3\") world"),
            "Hello   world"
        );
    }

    #[test]
    fn no_speak_wrapper_and_no_escaping() {
        assert_eq!(to_gemini("1 < 2 & 3 > 0"), "1 < 2 & 3 > 0");
    }
}
