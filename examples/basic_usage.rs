use speechmarkdown_rust::{Platform, SpeechMarkdownParser};

fn main() {
    let parser = SpeechMarkdownParser;

    // Test 1: Simple text
    println!("=== Test 1: Simple text ===");
    let input1 = "Hello world";
    match parser.to_text(input1) {
        Ok(text) => println!("Input: {}\nText: {}\n", input1, text),
        Err(e) => println!("Error: {:?}", e),
    }

    // Test 2: Breaks
    println!("=== Test 2: Breaks ===");
    let input2 = "Sample [2s] text [500ms] here";
    match parser.to_ssml(input2, Platform::AmazonAlexa) {
        Ok(ssml) => println!("Input: {}\nSSML: {}\n", input2, ssml),
        Err(e) => println!("Error: {:?}", e),
    }

    // Test 3: Emphasis
    println!("=== Test 3: Emphasis ===");
    let input3 = "++strong emphasis++ and +moderate emphasis+";
    match parser.to_ssml(input3, Platform::AmazonAlexa) {
        Ok(ssml) => println!("Input: {}\nSSML: {}\n", input3, ssml),
        Err(e) => println!("Error: {:?}", e),
    }

    // Test 4: Text modifiers
    println!("=== Test 4: Text modifiers ===");
    let input4 = "(text)[voice:\"Kendra\"] and (more)[emphasis:\"strong\"]";
    match parser.to_ssml(input4, Platform::AmazonAlexa) {
        Ok(ssml) => println!("Input: {}\nSSML: {}\n", input4, ssml),
        Err(e) => println!("Error: {:?}", e),
    }

    // Test 5: Audio
    println!("=== Test 5: Audio ===");
    let input5 = "Listen to this ![sound](\"https://example.com/audio.mp3\")";
    match parser.to_ssml(input5, Platform::AmazonAlexa) {
        Ok(ssml) => println!("Input: {}\nSSML: {}\n", input5, ssml),
        Err(e) => println!("Error: {:?}", e),
    }

    // Test 6: Complex sentence from test files
    println!("=== Test 6: Complex voice switching ===");
    let input6 = "Why do you keep switching voices (from one)[voice:\"Brian\"] to (the other)[voice:\"Kendra\"]?";
    match parser.to_text(input6) {
        Ok(text) => println!("Input: {}\nText: {}\n", input6, text),
        Err(e) => println!("Error: {:?}", e),
    }

    match parser.to_ssml(input6, Platform::AmazonAlexa) {
        Ok(ssml) => println!("SSML: {}\n", ssml),
        Err(e) => println!("Error: {:?}", e),
    }
}