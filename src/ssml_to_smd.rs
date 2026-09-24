use crate::error::{ParseError, Result};
use quick_xml::events::Event;
use quick_xml::Reader;

struct SsmlElement {
    name: String,
    attrs: Vec<(String, String)>,
    text_start: usize,
}

pub fn ssml_to_smd(ssml: &str) -> Result<String> {
    let stripped = strip_speak_tag(ssml);
    let mut reader = Reader::from_str(&stripped);
    reader.config_mut().trim_text(false);

    let mut result = String::new();
    let mut element_stack: Vec<SsmlElement> = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.name().local_name().as_ref().to_string();
                let attrs: Vec<(String, String)> = e
                    .attributes()
                    .filter_map(|a| a.ok())
                    .map(|a| {
                        let key = a.key.0.to_string();
                        let val = a.value.to_string();
                        (key, val)
                    })
                    .collect();

                element_stack.push(SsmlElement {
                    name,
                    attrs,
                    text_start: result.len(),
                });
            }
            Ok(Event::Empty(e)) => {
                let name = e.name().local_name().as_ref().to_string();
                let attrs: Vec<(String, String)> = e
                    .attributes()
                    .filter_map(|a| a.ok())
                    .map(|a| {
                        let key = a.key.0.to_string();
                        let val = a.value.to_string();
                        (key, val)
                    })
                    .collect();

                handle_self_closing(&name, &attrs, &mut result);
            }
            Ok(Event::Text(e)) => {
                let decoded = e.xml_content(quick_xml::XmlVersion::Explicit1_0);
                if !decoded.is_empty() {
                    result.push_str(&decoded);
                }
            }
            Ok(Event::End(_)) => {
                if let Some(elem) = element_stack.pop() {
                    handle_closing(&elem.name, &elem.attrs, elem.text_start, &mut result);
                }
            }
            Ok(Event::CData(e)) => {
                result.push_str(&e.into_inner());
            }
            Ok(Event::GeneralRef(e)) => {
                // &amp;/&lt;/&gt;/&#nn; arrive as separate events in 0.42
                // into_inner() is the bare ref name ("amp", "#60") —
                // re-wrap it for unescape.
                let raw = format!("&{};", e.into_inner());
                match quick_xml::escape::unescape(&raw) {
                    Ok(text) => result.push_str(&text),
                    Err(err) => {
                        return Err(ParseError::IoError(format!("entity decode error: {err}")));
                    }
                }
            }
            Err(e) => {
                return Err(ParseError::IoError(format!(
                    "SSML parse error at position {}: {:?}",
                    reader.error_position(),
                    e
                )));
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(result.trim().to_string())
}

fn strip_speak_tag(ssml: &str) -> String {
    let trimmed = ssml.trim();
    let mut result = trimmed;

    if result.starts_with("<speak") {
        if let Some(end_open) = result.find('>') {
            if result.ends_with("</speak>") {
                result = &result[end_open + 1..result.len() - 8];
            }
        }
    }

    result.to_string()
}

fn handle_self_closing(name: &str, attrs: &[(String, String)], result: &mut String) {
    match name {
        "break" => {
            if let Some(time) = get_attr(attrs, "time") {
                result.push_str(&format!("[{}]", time));
            } else if let Some(strength) = get_attr(attrs, "strength") {
                if ["none", "x-weak", "weak", "medium", "strong", "x-strong"]
                    .contains(&strength.as_str())
                {
                    result.push_str(&format!("[break:{}]", strength));
                } else {
                    result.push(' ');
                }
            } else {
                result.push(' ');
            }
        }
        "mark" => {
            if let Some(name_val) = get_attr(attrs, "name") {
                result.push_str(&format!("[mark:{}]", name_val));
            }
        }
        "audio" => {
            if let Some(src) = get_attr(attrs, "src") {
                result.push_str(&format!("![]({})", src));
            }
        }
        _ => {}
    }
}

fn handle_closing(name: &str, attrs: &[(String, String)], text_start: usize, result: &mut String) {
    match name {
        "emphasis" => {
            if let Some(level) = get_attr(attrs, "level") {
                let inner = extract_inner(text_start, result);
                match level.as_str() {
                    "strong" => {
                        result.push_str(&format!("++{}++", inner));
                    }
                    "moderate" => {
                        result.push_str(&format!("+{}+", inner));
                    }
                    "reduced" => {
                        result.push_str(&format!("-{}-", inner));
                    }
                    "none" => {
                        result.push_str(&format!("~{}~", inner));
                    }
                    _ => {
                        result.push_str(&inner);
                    }
                }
            }
        }
        "prosody" => {
            let mut modifiers = Vec::new();
            if let Some(rate) = get_attr(attrs, "rate") {
                if rate != "medium" {
                    modifiers.push(format!("rate:\"{}\"", rate));
                }
            }
            if let Some(pitch) = get_attr(attrs, "pitch") {
                if pitch != "medium" {
                    modifiers.push(format!("pitch:\"{}\"", pitch));
                }
            }
            if let Some(volume) = get_attr(attrs, "volume") {
                if volume != "medium" {
                    modifiers.push(format!("volume:\"{}\"", volume));
                }
            }
            if !modifiers.is_empty() {
                let inner = extract_inner(text_start, result);
                result.push_str(&format!("({})[{}]", inner, modifiers.join(";")));
            }
        }
        "voice" => {
            if let Some(voice_name) = get_attr(attrs, "name") {
                let inner = extract_inner(text_start, result);
                result.push_str(&format!("({})[voice:\"{}\"]", inner, voice_name));
            }
        }
        "lang" => {
            if let Some(lang) = get_attr(attrs, "xml:lang") {
                let inner = extract_inner(text_start, result);
                result.push_str(&format!("({})[lang:\"{}\"]", inner, lang));
            }
        }
        "phoneme" => {
            let alphabet = get_attr(attrs, "alphabet").unwrap_or_default();
            let ph = get_attr(attrs, "ph").unwrap_or_default();
            if alphabet.eq_ignore_ascii_case("ipa") && !ph.is_empty() {
                let inner = extract_inner(text_start, result);
                result.push_str(&format!("({})/{}", inner, ph));
            }
        }
        // Amazon's whispered effect (the local name quick-xml reports for
        // <amazon:effect>). Round-trips into the ElevenLabs dialects via
        // the whisper modifier.
        "effect" => {
            if get_attr(attrs, "name").is_some_and(|name| name.eq_ignore_ascii_case("whispered")) {
                let inner = extract_inner(text_start, result);
                if !inner.is_empty() {
                    result.push_str(&format!("({})[whisper]", inner));
                }
            }
        }
        // Azure's express-as (<mstts:express-as style="…">) → the generic
        // style modifier, which the ElevenLabs v3 dialect renders as an
        // audio tag and other platforms drop.
        "express-as" => {
            if let Some(style) = get_attr(attrs, "style") {
                let inner = extract_inner(text_start, result);
                if !inner.is_empty() {
                    result.push_str(&format!("({})[style:\"{}\"]", inner, style));
                }
            }
        }
        "sub" => {
            if let Some(alias) = get_attr(attrs, "alias") {
                let inner = extract_inner(text_start, result);
                result.push_str(&format!("{{{}}}", alias));
                if !inner.is_empty() && inner != alias {
                    result.push_str(&inner);
                }
            }
        }
        "desc" => {}
        "audio" => {
            if let Some(src) = get_attr(attrs, "src") {
                let inner = extract_inner(text_start, result);
                result.push_str(&format!("![{}]({})", inner, src));
            }
        }
        "say-as" => {
            let interpret_as = get_attr(attrs, "interpret-as").unwrap_or_default();
            let format_val = get_attr(attrs, "format").unwrap_or_default();
            let inner = extract_inner(text_start, result);

            let modifier = match interpret_as.as_str() {
                "characters" => "characters".to_string(),
                "number" | "cardinal" => "number".to_string(),
                "ordinal" => "ordinal".to_string(),
                "fraction" => "fraction".to_string(),
                "address" => "address".to_string(),
                "telephone" => "telephone".to_string(),
                "unit" => "unit".to_string(),
                "time" => {
                    if format_val.is_empty() {
                        "time".to_string()
                    } else {
                        format!("time:\"{}\"", format_val)
                    }
                }
                "date" => {
                    if format_val.is_empty() {
                        "date".to_string()
                    } else {
                        format!("date:\"{}\"", format_val)
                    }
                }
                "interjection" => "interjection".to_string(),
                "expletive" => "expletive".to_string(),
                other => other.to_string(),
            };

            if !modifier.is_empty() && !inner.is_empty() {
                result.push_str(&format!("({})[{}]", inner, modifier));
            } else if !inner.is_empty() {
                result.push_str(&inner);
            }
        }
        _ => {}
    }
}

fn extract_inner(text_start: usize, result: &mut String) -> String {
    if text_start >= result.len() {
        return String::new();
    }
    let inner = result[text_start..].to_string();
    result.truncate(text_start);
    inner.trim().to_string()
}

fn get_attr(attrs: &[(String, String)], key: &str) -> Option<String> {
    attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text() {
        let ssml = "<speak>Hello world</speak>";
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_break_time() {
        let ssml = r#"<speak>Hello <break time="2s"/> world</speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, "Hello [2s] world");
    }

    #[test]
    fn test_break_strength() {
        let ssml = r#"<speak>Hello <break strength="medium"/> world</speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, "Hello [break:medium] world");
    }

    #[test]
    fn test_emphasis_strong() {
        let ssml = r#"<speak><emphasis level="strong">word</emphasis></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, "++word++");
    }

    #[test]
    fn test_emphasis_moderate() {
        let ssml = r#"<speak><emphasis level="moderate">word</emphasis></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, "+word+");
    }

    #[test]
    fn test_emphasis_reduced() {
        let ssml = r#"<speak><emphasis level="reduced">word</emphasis></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, "-word-");
    }

    #[test]
    fn test_emphasis_none() {
        let ssml = r#"<speak><emphasis level="none">word</emphasis></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, "~word~");
    }

    #[test]
    fn test_prosody_rate() {
        let ssml = r#"<speak><prosody rate="slow">text</prosody></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, r#"(text)[rate:"slow"]"#);
    }

    #[test]
    fn test_prosody_rate_and_volume() {
        let ssml = r#"<speak><prosody rate="slow" volume="soft">text</prosody></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, r#"(text)[rate:"slow";volume:"soft"]"#);
    }

    #[test]
    fn test_voice() {
        let ssml = r#"<speak><voice name="Kendra">text</voice></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, r#"(text)[voice:"Kendra"]"#);
    }

    #[test]
    fn test_lang() {
        let ssml = r#"<speak><lang xml:lang="fr-FR">bonjour</lang></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, r#"(bonjour)[lang:"fr-FR"]"#);
    }

    #[test]
    fn test_phoneme_ipa() {
        let ssml = r#"<speak><phoneme alphabet="ipa" ph="ˈpi.kɑː.loʊ">piccolo</phoneme></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, "(piccolo)/ˈpi.kɑː.loʊ");
    }

    #[test]
    fn test_sub() {
        let ssml = r#"<speak><sub alias="AL">Al</sub></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, "{AL}Al");
    }

    #[test]
    fn test_audio_with_caption() {
        let ssml = r#"<speak><audio src="https://example.com/audio.mp3"><desc>sound effect</desc></audio></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert!(result.contains("sound effect"));
        assert!(result.contains("https://example.com/audio.mp3"));
    }

    #[test]
    fn test_audio_self_closing() {
        let ssml = r#"<speak><audio src="https://example.com/audio.mp3"/></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, "![](https://example.com/audio.mp3)");
    }

    #[test]
    fn test_mark() {
        let ssml = r#"<speak>Hello <mark name="mark1"/> world</speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, "Hello [mark:mark1] world");
    }

    #[test]
    fn test_say_as_characters() {
        let ssml = r#"<speak><say-as interpret-as="characters">ABC</say-as></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, "(ABC)[characters]");
    }

    #[test]
    fn test_say_as_date_with_format() {
        let ssml = r#"<speak><say-as interpret-as="date" format="mdy">01/02/2024</say-as></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, r#"(01/02/2024)[date:"mdy"]"#);
    }

    #[test]
    fn test_plain_text_no_speak_tag() {
        let result = ssml_to_smd("Hello world").unwrap();
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_mixed_content() {
        let ssml = r#"<speak>Hello <break time="500ms"/> <emphasis level="strong">world</emphasis></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, "Hello [500ms] ++world++");
    }

    #[test]
    fn test_xml_escaping() {
        let ssml = "<speak>A &amp; B &lt; C &gt; D</speak>";
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, "A & B < C > D");
    }

    #[test]
    fn test_amazon_whispered_effect() {
        let ssml =
            r#"<speak><amazon:effect name="whispered">it's a secret</amazon:effect></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, "(it's a secret)[whisper]");
    }

    #[test]
    fn test_amazon_other_effect_dropped() {
        let ssml = r#"<speak><amazon:effect name="damped">plain</amazon:effect></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, "plain");
    }

    #[test]
    fn test_mstts_express_as_style() {
        let ssml =
            r#"<speak><mstts:express-as style="cheerful">hello there</mstts:express-as></speak>"#;
        let result = ssml_to_smd(ssml).unwrap();
        assert_eq!(result, r#"(hello there)[style:"cheerful"]"#);
    }

    #[test]
    fn elevenlabs_round_trip_ssml_to_v3_dialect() {
        // The chain tts_speak_ssml uses: W3C SSML → SpeechMarkdown →
        // v3 audio-tag dialect.
        use crate::formatters::base::Platform;
        use crate::parser::SpeechMarkdownParser;
        let ssml = r#"<speak>Hello <break time="2s"/> <prosody rate="slow">world</prosody> <amazon:effect name="whispered">quietly</amazon:effect></speak>"#;
        let smd = ssml_to_smd(ssml).unwrap();
        let dialect = SpeechMarkdownParser::to_ssml(&smd, Platform::ElevenLabsV3).unwrap();
        assert_eq!(
            dialect,
            "Hello [long pause] [drawn out] world [whispers] quietly"
        );
    }
}
