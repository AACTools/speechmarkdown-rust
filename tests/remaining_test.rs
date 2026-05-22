use speechmarkdown_rust::Platform;
use speechmarkdown_rust::SpeechMarkdownParser;

fn check(name: &str, platform_str: &str, platform: Platform) {
    let test_dir = format!("tests/test-data/test-data/{}", name);
    let smd_path = format!("{}/{}.smd", test_dir, name);
    let expected_path = format!("{}/{}.{}.ssml", test_dir, name, platform_str);

    let input =
        std::fs::read_to_string(&smd_path).unwrap_or_else(|_| panic!("no smd: {}", smd_path));
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|_| panic!("no expected: {}", expected_path));

    let actual = SpeechMarkdownParser::to_ssml(input.trim(), platform).unwrap();

    if actual.trim() == expected.trim() {
        println!("{} [{}] PASS", name, platform_str);
    } else {
        println!("{} [{}] FAIL", name, platform_str);
        println!("  Input:    {:?}", input.trim());
        println!("  Expected: {:?}", expected.trim());
        println!("  Actual:   {:?}", actual.trim());
    }
}

#[test]
fn diagnose_remaining() {
    let cases = vec![
        // ipa-standard (Google only)
        ("ipa-standard", "google", Platform::GoogleAssistant),
        (
            "ipa-standard-alphabet-uk",
            "google",
            Platform::GoogleAssistant,
        ),
        (
            "ipa-standard-alphabet-us",
            "google",
            Platform::GoogleAssistant,
        ),
        // whisper (Google + Azure)
        ("whisper-standard", "google", Platform::GoogleAssistant),
        ("whisper-standard", "azure", Platform::MicrosoftAzure),
        // combo-whisper-emphasis (Google + Azure)
        (
            "combo-whisper-emphasis",
            "google",
            Platform::GoogleAssistant,
        ),
        ("combo-whisper-emphasis", "azure", Platform::MicrosoftAzure),
        // prosody multiple modifiers (all)
        (
            "prosody-multiple-modifiers vol + pitch + rate defaults",
            "alexa",
            Platform::AmazonAlexa,
        ),
        (
            "prosody-multiple-modifiers vol + pitch + rate defaults",
            "google",
            Platform::GoogleAssistant,
        ),
        (
            "prosody-multiple-modifiers vol + pitch + rate defaults",
            "azure",
            Platform::MicrosoftAzure,
        ),
        (
            "prosody-multiple-modifiers volume + pitch",
            "alexa",
            Platform::AmazonAlexa,
        ),
        (
            "prosody-multiple-modifiers volume + pitch",
            "google",
            Platform::GoogleAssistant,
        ),
        (
            "prosody-multiple-modifiers volume + pitch",
            "azure",
            Platform::MicrosoftAzure,
        ),
        (
            "prosody-multiple-modifiers volume + pitch + rate",
            "alexa",
            Platform::AmazonAlexa,
        ),
        (
            "prosody-multiple-modifiers volume + pitch + rate",
            "google",
            Platform::GoogleAssistant,
        ),
        (
            "prosody-multiple-modifiers volume + pitch + rate",
            "azure",
            Platform::MicrosoftAzure,
        ),
    ];

    for (name, platform_str, platform) in cases {
        check(name, platform_str, platform);
    }
}
