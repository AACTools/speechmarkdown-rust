use speechmarkdown_rust::{Platform, SpeechMarkdownParser};
use std::collections::HashMap;

fn debug_test(name: &str) -> (bool, HashMap<String, bool>) {
    let test_dir = std::path::Path::new("tests/test-data/test-data").join(name);
    let smd_file = test_dir.join(format!("{}.smd", name));
    if !smd_file.exists() {
        return (false, HashMap::new());
    }
    let input = std::fs::read_to_string(&smd_file)
        .unwrap()
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim_end_matches('\n')
        .to_string();

    let mut results = HashMap::new();

    let check = |platform: Platform, ext: &str| -> bool {
        let file = test_dir.join(format!("{}.{}", name, ext));
        if !file.exists() {
            return true;
        }
        let expected = std::fs::read_to_string(&file)
            .unwrap()
            .replace("\r\n", "\n")
            .replace('\r', "\n");
        match SpeechMarkdownParser::to_ssml(&input, platform) {
            Ok(actual) => actual.trim() == expected.trim(),
            Err(_) => false,
        }
    };

    results.insert("alexa".into(), check(Platform::AmazonAlexa, "alexa.ssml"));
    results.insert(
        "google".into(),
        check(Platform::GoogleAssistant, "google.ssml"),
    );
    results.insert(
        "azure".into(),
        check(Platform::MicrosoftAzure, "azure.ssml"),
    );

    let txt_file = test_dir.join(format!("{}.txt", name));
    let txt_pass = if txt_file.exists() {
        let expected = std::fs::read_to_string(&txt_file)
            .unwrap()
            .replace("\r\n", "\n")
            .replace('\r', "\n");
        match SpeechMarkdownParser::to_text(&input) {
            Ok(actual) => actual.trim() == expected.trim(),
            Err(_) => false,
        }
    } else {
        true
    };
    results.insert("txt".into(), txt_pass);

    let all_pass = results.values().all(|v| *v);
    (all_pass, results)
}

#[test]
fn debug_all_remaining() {
    let test_data_dir = std::path::Path::new("tests/test-data/test-data");
    let mut dirs: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(test_data_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            dirs.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    dirs.sort();

    let failures: Vec<String> = dirs
        .iter()
        .filter(|name| !debug_test(name).0)
        .cloned()
        .collect();

    println!("Failing tests by platform:");
    println!(
        "{:<55} {:>6} {:>6} {:>6} {:>4}",
        "Test", "alexa", "google", "azure", "txt"
    );
    for name in &failures {
        let (_, results) = debug_test(name);
        let a = if results.get("alexa").map_or(true, |v| *v) {
            "PASS"
        } else {
            "FAIL"
        };
        let g = if results.get("google").map_or(true, |v| *v) {
            "PASS"
        } else {
            "FAIL"
        };
        let z = if results.get("azure").map_or(true, |v| *v) {
            "PASS"
        } else {
            "FAIL"
        };
        let t = if results.get("txt").map_or(true, |v| *v) {
            "PASS"
        } else {
            "FAIL"
        };
        println!("{:<55} {:>6} {:>6} {:>6} {:>4}", name, a, g, z, t);
    }
    println!("\nTotal failures: {}", failures.len());

    // Debug specific test
    println!("\n\n=== DEBUG: sections-standard ===");
    let test_dir = std::path::Path::new("tests/test-data/test-data/sections-standard");
    let input = std::fs::read_to_string(test_dir.join("sections-standard.smd"))
        .unwrap()
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim_end_matches('\n')
        .to_string();
    let expected = std::fs::read_to_string(test_dir.join("sections-standard.alexa.ssml"))
        .unwrap()
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let actual = SpeechMarkdownParser::to_ssml(&input, Platform::AmazonAlexa).unwrap();
    println!("Input: {:?}", input);
    println!("Actual:   {:?}", actual.trim());
    println!("Expected: {:?}", expected.trim());
}
