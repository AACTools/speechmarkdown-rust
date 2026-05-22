use speechmarkdown_rust::{SpeechMarkdownParser, Platform};
use std::fs;

#[test]
fn test_break_strength_from_file() {
    let input = fs::read_to_string("tests/test-data/test-data/break-strength/break-strength.smd")
        .expect("Failed to read test file");

    // Normalize like the integration test does
    let input = input.replace("\r\n", "\n").replace('\r', "\n").trim_end_matches('\n').to_string();

    println!("Input: {:?}", input);

    let expected = fs::read_to_string("tests/test-data/test-data/break-strength/break-strength.alexa.ssml")
        .expect("Failed to read expected file");

    println!("Expected: {:?}", expected);

    match SpeechMarkdownParser::to_ssml(&input, Platform::AmazonAlexa) {
        Ok(actual) => {
            println!("Actual: {:?}", actual);
            let normalized_expected = expected.replace("\r\n", "\n").replace('\r', "\n");
            assert_eq!(actual.trim(), normalized_expected.trim());
        },
        Err(e) => panic!("SSML Error: {:?}", e),
    }
}