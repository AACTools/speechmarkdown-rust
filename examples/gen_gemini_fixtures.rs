//! Generates `.gemini.ssml` corpus expectations for the shared fixture
//! inputs, in the speechmarkdown-test-files layout.
//!
//! Run from the repo root:
//!   cargo run --example gen_gemini_fixtures -- <output-dir>
//!
//! Inputs mirror the elevenlabs-v3-* set where one exists (so dialects
//! are directly comparable); the rest cover gemini-specific policies
//! (backchannels as plain text, sound effects dropped, pause thresholds).
use speechmarkdown_rust::{Platform, SpeechMarkdownParser};
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = std::env::args()
        .nth(1)
        .expect("usage: gen_gemini_fixtures <output-dir>");
    let cases: Vec<(&str, &str)> = vec![
        ("gemini-break-short", "Sample [3s] speech [250ms] markdown"),
        (
            "gemini-break-strength",
            "None [break:\"none\"] x-weak [break:\"x-weak\"]\nweak [break:\"weak\"] medium [break:\"medium\"]\nstrong [break:\"strong\"] x-strong [break:\"x-strong\"]",
        ),
        (
            "gemini-emphasis-short",
            "This is ++important++ and +small+ and -muted- and ~flat~.",
        ),
        ("gemini-expressive", "He [laugh] and then [applause] left."),
        ("gemini-ipa-short", "Say (speech)/spitʃ/ clearly."),
        (
            "gemini-modifiers",
            "A (secret)[whisper], a (fast thing)[rate:\"fast\"], and a (loud thing)[volume:\"loud\"].",
        ),
        ("gemini-section-style", "#[whisper] Hello world"),
        ("gemini-sub-short", "The {AL}aluminum is recycled."),
        (
            "gemini-vocal-aliases",
            "[cheering] Go team! [whew] ... [throat-clear] anyway.",
        ),
        ("gemini-backchannels", "I [mhm] agree. [oh] really? Well [wow]."),
        (
            "gemini-pauses",
            "Hold on [750ms] alright... got it [1s] moving on [120ms] now.",
        ),
    ];

    for (name, input) in cases {
        let dir = Path::new(&out_dir).join(name);
        fs::create_dir_all(&dir).expect("create dir");
        fs::write(dir.join(format!("{name}.smd")), input).expect("write smd");
        let out = SpeechMarkdownParser::to_ssml(input, Platform::Gemini)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        fs::write(dir.join(format!("{name}.gemini.ssml")), out).expect("write expected");
        println!("wrote {name}");
    }
}
