use speechmarkdown_rust::{Platform, SpeechMarkdownParser};
use std::fs;
use std::path::Path;

#[test]
fn test_all_test_cases() {
    let test_data_dir = Path::new("tests/test-data");

    // Get all test directories
    let entries = fs::read_dir(test_data_dir).expect("Failed to read test-data directory");

    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let test_dir = entry.path();

        if !test_dir.is_dir() {
            continue;
        }

        let test_name = test_dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        println!("Running test: {}", test_name);

        // Read the .smd input file
        let smd_file = test_dir.join(format!("{}.smd", test_name));
        if !smd_file.exists() {
            continue;
        }

        let input = fs::read_to_string(&smd_file)
            .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", smd_file));

        // Test parsing
        let parser = SpeechMarkdownParser;
        let parse_result = parser.parse(&input);

        match parse_result {
            Ok(ast) => {
                // Test text output
                if let Ok(expected_text) = fs::read_to_string(test_dir.join(format!("{}.txt", test_name))) {
                    let text_result = parser.to_text(&input);
                    assert!(text_result.is_ok(), "Text formatting failed for {}", test_name);

                    let actual_text = text_result.unwrap();
                    assert_eq!(actual_text.trim(), expected_text.trim(),
                        "Text output mismatch for {}", test_name);
                }

                // Test SSML output for Amazon Alexa
                let alexa_file = test_dir.join(format!("{}.alexa.ssml", test_name));
                if alexa_file.exists() {
                    let expected_ssml = fs::read_to_string(&alexa_file)
                        .unwrap_or_else(|_| panic!("Failed to read Alexa SSML file: {:?}", alexa_file));

                    let ssml_result = parser.to_ssml(&input, Platform::AmazonAlexa);
                    assert!(ssml_result.is_ok(), "SSML formatting failed for {}", test_name);

                    let actual_ssml = ssml_result.unwrap();
                    assert_eq!(actual_ssml.trim(), expected_ssml.trim(),
                        "Alexa SSML output mismatch for {}", test_name);
                }

                // Test SSML output for Google Assistant
                let google_file = test_dir.join(format!("{}.google.ssml", test_name));
                if google_file.exists() {
                    let expected_ssml = fs::read_to_string(&google_file)
                        .unwrap_or_else(|_| panic!("Failed to read Google SSML file: {:?}", google_file));

                    let ssml_result = parser.to_ssml(&input, Platform::GoogleAssistant);
                    assert!(ssml_result.is_ok(), "Google SSML formatting failed for {}", test_name);

                    let actual_ssml = ssml_result.unwrap();
                    assert_eq!(actual_ssml.trim(), expected_ssml.trim(),
                        "Google SSML output mismatch for {}", test_name);
                }
            }
            Err(e) => {
                panic!("Parse failed for {}: {:?}", test_name, e);
            }
        }
    }
}

// Individual test cases for easier debugging
#[test]
fn test_break_short() {
    run_single_test("break-short");
}

#[test]
fn test_emphasis_short_strong() {
    run_single_test("emphasis-short-strong");
}

#[test]
fn test_text_modifier() {
    run_single_test("text-modifier");
}

fn run_single_test(test_name: &str) {
    let test_dir = Path::new("tests/test-data").join(test_name);
    let smd_file = test_dir.join(format!("{}.smd", test_name));

    let input = fs::read_to_string(&smd_file)
        .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", smd_file));

    let parser = SpeechMarkdownParser;
    let ast = parser.parse(&input)
        .expect("Parse failed");

    // Test text output if it exists
    let text_file = test_dir.join(format!("{}.txt", test_name));
    if text_file.exists() {
        let expected_text = fs::read_to_string(&text_file)
            .unwrap_or_else(|_| panic!("Failed to read text file: {:?}", text_file));

        let actual_text = parser.to_text(&input)
            .expect("Text formatting failed");

        assert_eq!(actual_text.trim(), expected_text.trim(),
            "Text output mismatch");
    }

    // Test SSML output if it exists
    let alexa_file = test_dir.join(format!("{}.alexa.ssml", test_name));
    if alexa_file.exists() {
        let expected_ssml = fs::read_to_string(&alexa_file)
            .unwrap_or_else(|_| panic!("Failed to read Alexa SSML file: {:?}", alexa_file));

        let actual_ssml = parser.to_ssml(&input, Platform::AmazonAlexa)
            .expect("SSML formatting failed");

        assert_eq!(actual_ssml.trim(), expected_ssml.trim(),
            "Alexa SSML output mismatch");
    }
}