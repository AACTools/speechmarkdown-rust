use speechmarkdown_rust::{Platform, SpeechMarkdownParser};
use std::fs;
use std::path::Path;

fn normalize_line_endings(text: &str) -> String {
    // Normalize all line endings to \n for comparison
    text.replace("\r\n", "\n").replace('\r', "\n")
}

#[test]
fn test_all_test_cases() {
    let test_data_dir = Path::new("tests/test-data/test-data");

    // Get all test directories
    let entries = fs::read_dir(test_data_dir).expect("Failed to read test-data directory");

    let mut passed = 0;
    let mut failed = 0;
    let mut failed_tests = Vec::new();

    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let test_dir = entry.path();

        if !test_dir.is_dir() {
            continue;
        }

        let test_name = test_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Fixtures that exercise the phonetic-translation feature; only
        // run them when that feature is on.
        #[cfg(not(feature = "phonetic-translation"))]
        if matches!(
            test_name,
            "xsampa-standard"
                | "xsampa-stress"
                | "praat-standard"
                | "sil-standard"
                | "branner-standard"
                | "combo-xsampa-emphasis"
        ) {
            continue;
        }

        // Read the .smd input file
        let smd_file = test_dir.join(format!("{}.smd", test_name));
        if !smd_file.exists() {
            continue;
        }

        let input = fs::read_to_string(&smd_file)
            .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", smd_file));

        // Normalize line endings in the input before parsing
        let input = normalize_line_endings(&input)
            .trim_end_matches('\n')
            .to_string();

        // Test parsing
        let parse_result = SpeechMarkdownParser::parse(&input);

        let test_passed = match parse_result {
            Ok(_ast) => {
                let mut all_checks_passed = true;

                // Test text output
                if let Ok(expected_text) =
                    fs::read_to_string(test_dir.join(format!("{}.txt", test_name)))
                {
                    let text_result = SpeechMarkdownParser::to_text(&input);
                    if text_result.is_err() {
                        all_checks_passed = false;
                    } else if let Ok(actual_text) = text_result {
                        if actual_text.trim() != normalize_line_endings(expected_text.trim()) {
                            all_checks_passed = false;
                        }
                    }
                }

                // Test SSML output for Amazon Alexa
                let alexa_file = test_dir.join(format!("{}.alexa.ssml", test_name));
                if alexa_file.exists() {
                    let expected_ssml = fs::read_to_string(&alexa_file).unwrap_or_else(|_| {
                        panic!("Failed to read Alexa SSML file: {:?}", alexa_file)
                    });

                    let ssml_result = SpeechMarkdownParser::to_ssml(&input, Platform::AmazonAlexa);
                    if ssml_result.is_err() {
                        all_checks_passed = false;
                    } else if let Ok(actual_ssml) = ssml_result {
                        if actual_ssml.trim() != normalize_line_endings(expected_ssml.trim()) {
                            all_checks_passed = false;
                        }
                    }
                }

                // Test SSML output for Google Assistant
                let google_file = test_dir.join(format!("{}.google.ssml", test_name));
                if google_file.exists() {
                    let expected_ssml = fs::read_to_string(&google_file).unwrap_or_else(|_| {
                        panic!("Failed to read Google SSML file: {:?}", google_file)
                    });

                    let ssml_result =
                        SpeechMarkdownParser::to_ssml(&input, Platform::GoogleAssistant);
                    if ssml_result.is_err() {
                        all_checks_passed = false;
                    } else if let Ok(actual_ssml) = ssml_result {
                        if actual_ssml.trim() != normalize_line_endings(expected_ssml.trim()) {
                            all_checks_passed = false;
                        }
                    }
                }

                // Test prompt markup output for ElevenLabs (pre-v3 dialect)
                let elevenlabs_file = test_dir.join(format!("{}.elevenlabs.ssml", test_name));
                if elevenlabs_file.exists() {
                    let expected = fs::read_to_string(&elevenlabs_file).unwrap_or_else(|_| {
                        panic!("Failed to read ElevenLabs file: {:?}", elevenlabs_file)
                    });

                    let result = SpeechMarkdownParser::to_ssml(&input, Platform::ElevenLabs);
                    if result.is_err() {
                        all_checks_passed = false;
                    } else if let Ok(actual) = result {
                        if actual.trim() != normalize_line_endings(expected.trim()) {
                            all_checks_passed = false;
                        }
                    }
                }

                // Test audio-tag output for ElevenLabs v3 (our own fixture
                // set — the speechmarkdown-js reference has no v3 platform)
                let elevenlabs_v3_file = test_dir.join(format!("{}.elevenlabs-v3.ssml", test_name));
                if elevenlabs_v3_file.exists() {
                    let expected = fs::read_to_string(&elevenlabs_v3_file).unwrap_or_else(|_| {
                        panic!(
                            "Failed to read ElevenLabs v3 file: {:?}",
                            elevenlabs_v3_file
                        )
                    });

                    let result = SpeechMarkdownParser::to_ssml(&input, Platform::ElevenLabsV3);
                    if result.is_err() {
                        all_checks_passed = false;
                    } else if let Ok(actual) = result {
                        if actual.trim() != normalize_line_endings(expected.trim()) {
                            all_checks_passed = false;
                        }
                    }
                }

                all_checks_passed
            }
            Err(_e) => false,
        };

        if test_passed {
            passed += 1;
        } else {
            failed += 1;
            failed_tests.push(test_name.to_string());
        }
    }

    println!("=== Test Summary ===");
    println!("Total tests: {}", passed + failed);
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!(
        "Pass rate: {:.1}%",
        (passed as f64 / (passed + failed) as f64) * 100.0
    );

    if failed > 0 {
        println!("\n=== All Failed Tests ===");
        for test in &failed_tests {
            println!("  - {}", test);
        }
        // Zero tolerance: this suite is the regression net for the
        // enforced fixture families (.txt, .alexa.ssml, .google.ssml,
        // .elevenlabs.ssml). Any failure should fail CI — a lenient
        // threshold would let a broken formatter slip through while
        // enough unrelated cases still pass.
        panic!(
            "{}/{} corpus tests failed: {:?}",
            failed,
            passed + failed,
            failed_tests
        );
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
fn test_address_standard() {
    run_single_test("address-standard");
}

fn run_single_test(test_name: &str) {
    let test_dir = Path::new("tests/test-data/test-data").join(test_name);
    let smd_file = test_dir.join(format!("{}.smd", test_name));

    let input = fs::read_to_string(&smd_file)
        .unwrap_or_else(|_| panic!("Failed to read test file: {:?}", smd_file))
        .trim_end_matches(['\r', '\n'])
        .to_string();

    let _ast = SpeechMarkdownParser::parse(&input).expect("Parse failed");

    // Test text output if it exists
    let text_file = test_dir.join(format!("{}.txt", test_name));
    if text_file.exists() {
        let expected_text = fs::read_to_string(&text_file)
            .unwrap_or_else(|_| panic!("Failed to read text file: {:?}", text_file));

        let actual_text = SpeechMarkdownParser::to_text(&input).expect("Text formatting failed");

        assert_eq!(
            actual_text.trim(),
            normalize_line_endings(expected_text.trim()),
        );
    }

    // Test SSML output if it exists
    let alexa_file = test_dir.join(format!("{}.alexa.ssml", test_name));
    if alexa_file.exists() {
        let expected_ssml = fs::read_to_string(&alexa_file)
            .unwrap_or_else(|_| panic!("Failed to read Alexa SSML file: {:?}", alexa_file));

        let actual_ssml = SpeechMarkdownParser::to_ssml(&input, Platform::AmazonAlexa)
            .expect("SSML formatting failed");

        assert_eq!(
            actual_ssml.trim(),
            normalize_line_endings(expected_ssml.trim()),
            "Alexa SSML output mismatch"
        );
    }
}
