use crate::formatters::base::Platform;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SsmlCapability {
    pub element: String,
    pub description: String,
    pub attributes: Vec<String>,
    pub speech_markdown_syntax: Vec<String>,
    pub example: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlatformCapabilities {
    pub platform: String,
    pub ssml_elements: Vec<SsmlCapability>,
    pub unsupported: Vec<String>,
}

pub fn get_supported_ssml(platform: Platform) -> PlatformCapabilities {
    match platform {
        Platform::AmazonAlexa => amazon_alexa_capabilities(),
        Platform::GoogleAssistant => google_assistant_capabilities(),
        Platform::MicrosoftAzure => microsoft_azure_capabilities(),
        Platform::Apple => apple_capabilities(),
        Platform::W3c => w3c_capabilities(),
        Platform::SamsungBixby => samsung_bixby_capabilities(),
        Platform::ElevenLabs => elevenlabs_capabilities(),
        Platform::ElevenLabsV3 => elevenlabs_v3_capabilities(),
        Platform::IbmWatson => ibm_watson_capabilities(),
    }
}

fn break_element() -> SsmlCapability {
    SsmlCapability {
        element: "break".into(),
        description: "Insert a pause".into(),
        attributes: vec!["time".into(), "strength".into()],
        speech_markdown_syntax: vec!["[2s]".into(), "[500ms]".into(), "[break:strong]".into()],
        example: "Hello [2s] world".into(),
    }
}

fn emphasis_element() -> SsmlCapability {
    SsmlCapability {
        element: "emphasis".into(),
        description: "Emphasize text".into(),
        attributes: vec!["level".into()],
        speech_markdown_syntax: vec![
            "+word+".into(),
            "++word++".into(),
            "-word-".into(),
            "~word~".into(),
        ],
        example: "++important++".into(),
    }
}

fn prosody_element() -> SsmlCapability {
    SsmlCapability {
        element: "prosody".into(),
        description: "Control rate, pitch, and volume".into(),
        attributes: vec!["rate".into(), "pitch".into(), "volume".into()],
        speech_markdown_syntax: vec![
            "(text)[rate:\"slow\"]".into(),
            "(text)[pitch:\"high\"]".into(),
            "(text)[volume:\"soft\"]".into(),
        ],
        example: "(read this)[rate:\"fast\";volume:\"loud\"]".into(),
    }
}

fn audio_element() -> SsmlCapability {
    SsmlCapability {
        element: "audio".into(),
        description: "Play an audio file".into(),
        attributes: vec!["src".into()],
        speech_markdown_syntax: vec!["![caption](url)".into(), "![](url)".into()],
        example: "![sound](https://example.com/audio.mp3)".into(),
    }
}

fn say_as_element(sub_type: &str, _interpret_as: &str) -> SsmlCapability {
    let (syntax, desc) = match sub_type {
        "characters" => ("(ABC)[characters]", "Spell out characters"),
        "number" => ("(42)[number]", "Read as cardinal number"),
        "ordinal" => ("(1)[ordinal]", "Read as ordinal (first, second)"),
        "fraction" => ("(1/2)[fraction]", "Read as fraction"),
        "telephone" => ("(555-1234)[telephone]", "Read as phone number"),
        "address" => ("(123 Main St)[address]", "Read as address"),
        "unit" => ("(5kg)[unit]", "Read as unit of measurement"),
        "time" => ("(2:30)[time:\"hms12\"]", "Read as time"),
        "date" => ("(01/02/2024)[date:\"mdy\"]", "Read as date"),
        "interjection" => ("(wow)[interjection]", "Read as interjection"),
        "expletive" => ("(word)[expletive]", "Bleep/censor word"),
        _ => ("", "Unknown"),
    };
    SsmlCapability {
        element: "say-as".into(),
        description: desc.into(),
        attributes: vec!["interpret-as".into(), "format".into()],
        speech_markdown_syntax: vec![syntax.into()],
        example: syntax.into(),
    }
}

fn sub_element() -> SsmlCapability {
    SsmlCapability {
        element: "sub".into(),
        description: "Substitute pronunciation".into(),
        attributes: vec!["alias".into()],
        speech_markdown_syntax: vec!["{alias}text".into()],
        example: "{AL}aluminum".into(),
    }
}

fn mark_element() -> SsmlCapability {
    SsmlCapability {
        element: "mark".into(),
        description: "Insert a named marker".into(),
        attributes: vec!["name".into()],
        speech_markdown_syntax: vec!["[mark:name]".into()],
        example: "Hello [mark:chapter1] world".into(),
    }
}

fn phoneme_element() -> SsmlCapability {
    SsmlCapability {
        element: "phoneme".into(),
        description: "Custom pronunciation (IPA)".into(),
        attributes: vec!["alphabet".into(), "ph".into()],
        speech_markdown_syntax: vec!["(text)/phoneme".into()],
        example: "(piccolo)/ˈpi.kɑː.loʊ".into(),
    }
}

fn voice_element() -> SsmlCapability {
    SsmlCapability {
        element: "voice".into(),
        description: "Switch to a different voice".into(),
        attributes: vec!["name".into()],
        speech_markdown_syntax: vec!["(text)[voice:\"Kendra\"]".into()],
        example: "(hello)[voice:\"Brian\"]".into(),
    }
}

fn lang_element() -> SsmlCapability {
    SsmlCapability {
        element: "lang".into(),
        description: "Set language".into(),
        attributes: vec!["xml:lang".into()],
        speech_markdown_syntax: vec!["(text)[lang:\"fr-FR\"]".into()],
        example: "(bonjour)[lang:\"fr-FR\"]".into(),
    }
}

fn amazon_alexa_capabilities() -> PlatformCapabilities {
    PlatformCapabilities {
        platform: "amazon-alexa".into(),
        ssml_elements: vec![
            break_element(),
            emphasis_element(),
            prosody_element(),
            audio_element(),
            sub_element(),
            phoneme_element(),
            mark_element(),
            lang_element(),
            voice_element(),
            say_as_element("characters", "characters"),
            say_as_element("number", "number"),
            say_as_element("ordinal", "ordinal"),
            say_as_element("fraction", "fraction"),
            say_as_element("telephone", "telephone"),
            say_as_element("address", "address"),
            say_as_element("unit", "unit"),
            say_as_element("time", "time"),
            say_as_element("date", "date"),
            say_as_element("interjection", "interjection"),
            say_as_element("expletive", "expletive"),
            SsmlCapability {
                element: "amazon:effect".into(),
                description: "Whisper effect".into(),
                attributes: vec!["name".into()],
                speech_markdown_syntax: vec!["(text)[whisper]".into()],
                example: "(hello)[whisper]".into(),
            },
            SsmlCapability {
                element: "amazon:emotion".into(),
                description: "Express emotion (excited/disappointed)".into(),
                attributes: vec!["name".into(), "intensity".into()],
                speech_markdown_syntax: vec![
                    "#[excited] text".into(),
                    "#[disappointed] text".into(),
                ],
                example: "#[excited] Great news!".into(),
            },
            SsmlCapability {
                element: "amazon:domain".into(),
                description: "Switch to news or music domain".into(),
                attributes: vec!["name".into()],
                speech_markdown_syntax: vec!["#[newscaster] text".into(), "#[dj] text".into()],
                example: "#[newscaster] Breaking news today".into(),
            },
        ],
        unsupported: vec!["google:style".into(), "mstts:express-as".into()],
    }
}

fn google_assistant_capabilities() -> PlatformCapabilities {
    PlatformCapabilities {
        platform: "google-assistant".into(),
        ssml_elements: vec![
            break_element(),
            emphasis_element(),
            prosody_element(),
            audio_element(),
            sub_element(),
            mark_element(),
            say_as_element("characters", "characters"),
            say_as_element("number", "number"),
            say_as_element("ordinal", "ordinal"),
            say_as_element("fraction", "fraction"),
            say_as_element("telephone", "telephone"),
            say_as_element("address", "address"),
            say_as_element("unit", "unit"),
            say_as_element("time", "time"),
            say_as_element("date", "date"),
            say_as_element("interjection", "interjection"),
            say_as_element("expletive", "expletive"),
            SsmlCapability {
                element: "google:style".into(),
                description: "Google speaking style".into(),
                attributes: vec!["name".into()],
                speech_markdown_syntax: vec!["(text)[style:\"name\"]".into()],
                example: "(hello)[style:\"cheerful\"]".into(),
            },
        ],
        unsupported: vec![
            "voice".into(),
            "lang".into(),
            "phoneme".into(),
            "amazon:effect".into(),
            "amazon:emotion".into(),
            "amazon:domain".into(),
            "mstts:express-as".into(),
            "excited section".into(),
            "disappointed section".into(),
        ],
    }
}

fn microsoft_azure_capabilities() -> PlatformCapabilities {
    PlatformCapabilities {
        platform: "microsoft-azure".into(),
        ssml_elements: vec![
            break_element(),
            prosody_element(),
            audio_element(),
            sub_element(),
            phoneme_element(),
            mark_element(),
            lang_element(),
            voice_element(),
            say_as_element("characters", "characters"),
            say_as_element("number", "cardinal"),
            say_as_element("ordinal", "ordinal"),
            say_as_element("fraction", "fraction"),
            say_as_element("telephone", "telephone"),
            say_as_element("address", "address"),
            say_as_element("unit", "unit"),
            say_as_element("time", "time"),
            say_as_element("date", "date"),
            say_as_element("interjection", "interjection"),
            say_as_element("expletive", "expletive"),
            SsmlCapability {
                element: "mstts:express-as".into(),
                description: "Express emotion/style (42 styles)".into(),
                attributes: vec!["style".into()],
                speech_markdown_syntax: vec![
                    "#[cheerful] text".into(),
                    "#[sad] text".into(),
                    "#[angry] text".into(),
                    "(text)[excited]".into(),
                    "(text)[disappointed]".into(),
                ],
                example: "#[cheerful] Hello there!".into(),
            },
            SsmlCapability {
                element: "prosody (whisper)".into(),
                description: "Whisper via prosody (rate:slow + volume:x-soft)".into(),
                attributes: vec!["rate".into(), "volume".into()],
                speech_markdown_syntax: vec!["(text)[whisper]".into()],
                example: "(hello)[whisper]".into(),
            },
        ],
        unsupported: vec![
            "emphasis (not supported by Azure)".into(),
            "amazon:effect".into(),
            "amazon:emotion".into(),
            "amazon:domain".into(),
        ],
    }
}

fn apple_capabilities() -> PlatformCapabilities {
    PlatformCapabilities {
        platform: "apple".into(),
        ssml_elements: vec![
            break_element(),
            emphasis_element(),
            prosody_element(),
            audio_element(),
            sub_element(),
            phoneme_element(),
            mark_element(),
            lang_element(),
            voice_element(),
            say_as_element("characters", "characters"),
            say_as_element("number", "number"),
            say_as_element("ordinal", "ordinal"),
            say_as_element("date", "date"),
            say_as_element("time", "time"),
        ],
        unsupported: vec![
            "amazon:effect".into(),
            "amazon:emotion".into(),
            "amazon:domain".into(),
            "mstts:express-as".into(),
            "google:style".into(),
        ],
    }
}

fn w3c_capabilities() -> PlatformCapabilities {
    PlatformCapabilities {
        platform: "w3c".into(),
        ssml_elements: vec![
            break_element(),
            emphasis_element(),
            prosody_element(),
            audio_element(),
            sub_element(),
            phoneme_element(),
            mark_element(),
            lang_element(),
            voice_element(),
            say_as_element("characters", "characters"),
            say_as_element("number", "number"),
            say_as_element("ordinal", "ordinal"),
            say_as_element("fraction", "fraction"),
            say_as_element("telephone", "telephone"),
            say_as_element("address", "address"),
            say_as_element("unit", "unit"),
            say_as_element("time", "time"),
            say_as_element("date", "date"),
            say_as_element("interjection", "interjection"),
            say_as_element("expletive", "expletive"),
        ],
        unsupported: vec![
            "amazon:effect".into(),
            "amazon:emotion".into(),
            "amazon:domain".into(),
            "mstts:express-as".into(),
            "google:style".into(),
        ],
    }
}

fn samsung_bixby_capabilities() -> PlatformCapabilities {
    PlatformCapabilities {
        platform: "samsung-bixby".into(),
        ssml_elements: vec![
            break_element(),
            emphasis_element(),
            prosody_element(),
            audio_element(),
            sub_element(),
            mark_element(),
            say_as_element("characters", "characters"),
            say_as_element("number", "number"),
            say_as_element("ordinal", "ordinal"),
            say_as_element("date", "date"),
            say_as_element("time", "time"),
        ],
        unsupported: vec![
            "voice".into(),
            "lang".into(),
            "phoneme".into(),
            "amazon:effect".into(),
            "amazon:emotion".into(),
            "amazon:domain".into(),
            "mstts:express-as".into(),
            "google:style".into(),
        ],
    }
}

fn elevenlabs_capabilities() -> PlatformCapabilities {
    // Pre-v3 prompt markup (eleven_multilingual_v2 / flash_v2_5 /
    // flash_v2 / turbo_v2): no SSML document, but two inline tags are
    // parsed. Everything else degrades to plain text.
    PlatformCapabilities {
        platform: "elevenlabs".into(),
        ssml_elements: vec![
            SsmlCapability {
                element: "break".into(),
                description: "Insert a pause (pre-v3 models only; max 3s; \
                              eleven_v3 does not parse break tags)"
                    .into(),
                attributes: vec!["time".into()],
                speech_markdown_syntax: vec![
                    "[2s]".into(),
                    "[500ms]".into(),
                    "[break:strong]".into(),
                ],
                example: "Hello [2s] world".into(),
            },
            SsmlCapability {
                element: "phoneme".into(),
                description: "Custom pronunciation (eleven_flash_v2 / \
                              eleven_turbo_v2 only, English only)"
                    .into(),
                attributes: vec!["alphabet".into(), "ph".into()],
                speech_markdown_syntax: vec![
                    "(text)[ipa:\"pɪkəloʊ\"]".into(),
                    "(text)/pɪkəloʊ/".into(),
                ],
                example: "(piccolo)/pɪkəloʊ/".into(),
            },
        ],
        unsupported: vec![
            "emphasis (no equivalent; degrades to text)".into(),
            "prosody (rate maps to the API voice_settings.speed; use the \
             elevenlabs-v3 dialect for tag-based control)"
                .into(),
            "say-as (text normalization is built in)".into(),
            "sub (use pronunciation dictionaries or phonetic spelling)".into(),
            "audio (no equivalent)".into(),
            "mark (no equivalent)".into(),
            "voice (switch via the API voice_id parameter)".into(),
            "lang (no equivalent)".into(),
            "audio tags (eleven_v3 only)".into(),
            "amazon:effect".into(),
            "amazon:emotion".into(),
            "amazon:domain".into(),
            "mstts:express-as".into(),
            "google:style".into(),
        ],
    }
}

fn elevenlabs_v3_capabilities() -> PlatformCapabilities {
    // Eleven v3 audio-tag dialect: no SSML at all. Bracketed
    // natural-language tags are prompts interpreted by the model —
    // best-effort, voice-dependent, and directional from their insertion
    // point (no span semantics).
    PlatformCapabilities {
        platform: "elevenlabs-v3".into(),
        ssml_elements: vec![
            SsmlCapability {
                element: "[audio tag]".into(),
                description: "Natural-language performance direction in \
                              brackets; open-ended (emotions, delivery, \
                              reactions, sound effects, accents)"
                    .into(),
                attributes: vec![],
                speech_markdown_syntax: vec![
                    "[laugh]".into(),
                    "#[excited] text".into(),
                    "(text)[whisper]".into(),
                ],
                example: "[whispers] It's a secret".into(),
            },
            SsmlCapability {
                element: "[pause] family".into(),
                description: "Pauses via [short pause] / [pause] / [long \
                              pause] or ellipses; approximate — no exact \
                              durations on v3"
                    .into(),
                attributes: vec![],
                speech_markdown_syntax: vec![
                    "[500ms]".into(),
                    "[2s]".into(),
                    "[break:strong]".into(),
                ],
                example: "Hello [2s] world".into(),
            },
            SsmlCapability {
                element: "inline IPA".into(),
                description: "Native IPA wrapped in quotes and slashes".into(),
                attributes: vec![],
                speech_markdown_syntax: vec!["(speech)/spitʃ/".into()],
                example: "(speech)/spitʃ/".into(),
            },
            SsmlCapability {
                element: "emphasis tags".into(),
                description: "Emphasis via [emphasized] / [stress on next \
                              word] / [understated] (capitalization also \
                              works but mutates the text)"
                    .into(),
                attributes: vec![],
                speech_markdown_syntax: vec!["++word++".into(), "+word+".into()],
                example: "++important++".into(),
            },
        ],
        unsupported: vec![
            "break (eleven_v3 does not parse SSML break tags)".into(),
            "phoneme (use inline IPA)".into(),
            "prosody (rate maps to the API voice_settings.speed; \
             tempo/volume map to [rushed]/[drawn out]/[softly]/[loudly] tags)"
                .into(),
            "say-as (text normalization is built in)".into(),
            "sub (alias is spoken instead of the text)".into(),
            "audio (no equivalent)".into(),
            "mark (no equivalent)".into(),
            "voice (switch via the API voice_id parameter or Text to \
             Dialogue)"
                .into(),
            "lang (no equivalent)".into(),
            "amazon:effect".into(),
            "amazon:emotion".into(),
            "amazon:domain".into(),
            "mstts:express-as".into(),
            "google:style".into(),
        ],
    }
}

fn ibm_watson_capabilities() -> PlatformCapabilities {
    PlatformCapabilities {
        platform: "ibm-watson".into(),
        ssml_elements: vec![
            break_element(),
            emphasis_element(),
            prosody_element(),
            audio_element(),
            sub_element(),
            mark_element(),
            say_as_element("characters", "characters"),
            say_as_element("number", "number"),
            say_as_element("ordinal", "ordinal"),
            say_as_element("date", "date"),
            say_as_element("time", "time"),
        ],
        unsupported: vec![
            "voice".into(),
            "lang".into(),
            "phoneme".into(),
            "amazon:effect".into(),
            "amazon:emotion".into(),
            "amazon:domain".into(),
            "mstts:express-as".into(),
            "google:style".into(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_platforms_have_capabilities() {
        for platform in [
            Platform::AmazonAlexa,
            Platform::GoogleAssistant,
            Platform::MicrosoftAzure,
            Platform::Apple,
            Platform::W3c,
            Platform::SamsungBixby,
            Platform::ElevenLabs,
            Platform::ElevenLabsV3,
            Platform::IbmWatson,
        ] {
            let caps = get_supported_ssml(platform);
            assert!(
                !caps.ssml_elements.is_empty(),
                "{:?} has no elements",
                platform
            );
            assert!(!caps.platform.is_empty());
        }
    }

    #[test]
    fn test_all_platforms_have_break() {
        for platform in [
            Platform::AmazonAlexa,
            Platform::GoogleAssistant,
            Platform::MicrosoftAzure,
            Platform::Apple,
            Platform::W3c,
            Platform::SamsungBixby,
            Platform::ElevenLabs,
            Platform::IbmWatson,
        ] {
            let caps = get_supported_ssml(platform);
            assert!(
                caps.ssml_elements.iter().any(|e| e.element == "break"),
                "{:?} missing break",
                platform
            );
        }

        // Eleven v3 does not parse <break> at all; pauses are audio tags.
        let v3 = get_supported_ssml(Platform::ElevenLabsV3);
        assert!(v3
            .ssml_elements
            .iter()
            .any(|e| e.element == "[pause] family"));
        assert!(v3.unsupported.iter().any(|u| u.starts_with("break")));
    }

    #[test]
    fn test_alexa_has_emotion() {
        let caps = get_supported_ssml(Platform::AmazonAlexa);
        assert!(caps
            .ssml_elements
            .iter()
            .any(|e| e.element == "amazon:emotion"));
    }

    #[test]
    fn test_azure_has_express_as() {
        let caps = get_supported_ssml(Platform::MicrosoftAzure);
        assert!(caps
            .ssml_elements
            .iter()
            .any(|e| e.element == "mstts:express-as"));
    }

    #[test]
    fn test_google_no_voice() {
        let caps = get_supported_ssml(Platform::GoogleAssistant);
        assert!(caps.unsupported.contains(&"voice".to_string()));
    }

    #[test]
    fn test_azure_no_emphasis() {
        let caps = get_supported_ssml(Platform::MicrosoftAzure);
        assert!(caps.unsupported.iter().any(|u| u.contains("emphasis")));
    }

    #[test]
    fn test_serialization() {
        let caps = get_supported_ssml(Platform::AmazonAlexa);
        let json = serde_json::to_string(&caps).unwrap();
        assert!(json.contains("amazon:emotion"));
        let deserialized: PlatformCapabilities = serde_json::from_str(&json).unwrap();
        assert_eq!(caps, deserialized);
    }
}
