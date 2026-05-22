use speechmarkdown_rust::{Platform, SpeechMarkdownParser};
use std::fs;
use std::path::Path;

fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn debug_test(name: &str) {
    let test_dir = Path::new("tests/test-data/test-data").join(name);
    let smd_file = test_dir.join(format!("{}.smd", name));
    if !smd_file.exists() {
        println!("=== {} === SMD file not found", name);
        return;
    }
    let input = fs::read_to_string(&smd_file)
        .unwrap()
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim_end_matches('\n')
        .to_string();

    println!("=== {} ===", name);
    println!("Input: {:?}", input);

    for (platform, ext) in [
        (Platform::AmazonAlexa, "alexa.ssml"),
        (Platform::GoogleAssistant, "google.ssml"),
        (Platform::MicrosoftAzure, "azure.ssml"),
    ] {
        let file = test_dir.join(format!("{}.{}", name, ext));
        if !file.exists() {
            continue;
        }
        let expected = normalize(&fs::read_to_string(&file).unwrap());
        match SpeechMarkdownParser::to_ssml(&input, platform) {
            Ok(actual) => {
                if actual.trim() != expected.trim() {
                    println!("[{}] MISMATCH:", ext);
                    println!("  Actual:   {:?}", actual.trim());
                    println!("  Expected: {:?}", expected.trim());
                } else {
                    println!("[{}] PASS", ext);
                }
            }
            Err(e) => println!("[{}] ERROR: {:?}", ext, e),
        }
    }

    let txt_file = test_dir.join(format!("{}.txt", name));
    if txt_file.exists() {
        let expected = normalize(&fs::read_to_string(&txt_file).unwrap());
        match SpeechMarkdownParser::to_text(&input) {
            Ok(actual) => {
                if actual.trim() != expected.trim() {
                    println!("[txt] MISMATCH:");
                    println!("  Actual:   {:?}", actual.trim());
                    println!("  Expected: {:?}", expected.trim());
                } else {
                    println!("[txt] PASS");
                }
            }
            Err(e) => println!("[txt] ERROR: {:?}", e),
        }
    }
    println!();
}

#[test]
fn diagnose_failures() {
    let tests = [
        "azure-section-angry",
        "azure-section-friendly",
        "bare-ipa",
        "ipa-short",
        "combo-emphasis-prosody",
        "combo-voice-prosody",
        "combo-ipa-emphasis",
        "disappointed-standard invalid intensity",
        "excited-standard invalid intensity",
        "disappointed-section normal to disappointed intensities to normal",
        "excited-section normal to excited intensities to normal",
        "prosody-multiple-modifiers rate + pitch",
        "prosody-multiple-modifiers rate + volume",
        "multiple-modifiers-same-text",
        "voice-azure-displayname",
        "sections-standard voice section on same line",
    ];
    for t in &tests {
        debug_test(t);
    }
}
