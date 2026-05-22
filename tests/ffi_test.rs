use std::ffi::{CStr, CString};

use speechmarkdown_rust::ffi;
use speechmarkdown_rust::parser::SpeechMarkdownParser;
use speechmarkdown_rust::Platform;

#[test]
fn test_ffi_to_ssml_alexa() {
    let input = CString::new("Hello (world)[emphasis:\"strong\"]").unwrap();
    let platform = CString::new("amazon-alexa").unwrap();

    let result = ffi::speechmarkdown_to_ssml(input.as_ptr(), platform.as_ptr());
    assert!(!result.is_null());

    let output = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
    assert!(output.contains("<speak>"));
    assert!(output.contains("<emphasis level=\"strong\">world</emphasis>"));

    ffi::speechmarkdown_free(result);
}

#[test]
fn test_ffi_to_ssml_google() {
    let input = CString::new("Hello (world)[rate:\"fast\"]").unwrap();
    let platform = CString::new("google-assistant").unwrap();

    let result = ffi::speechmarkdown_to_ssml(input.as_ptr(), platform.as_ptr());
    assert!(!result.is_null());

    let output = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
    assert!(output.contains("<prosody rate=\"fast\">world</prosody>"));

    ffi::speechmarkdown_free(result);
}

#[test]
fn test_ffi_to_ssml_azure() {
    let input = CString::new("Hello (world)[whisper]").unwrap();
    let platform = CString::new("microsoft-azure").unwrap();

    let result = ffi::speechmarkdown_to_ssml(input.as_ptr(), platform.as_ptr());
    assert!(!result.is_null());

    let output = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
    assert!(output.contains("<prosody"));

    ffi::speechmarkdown_free(result);
}

#[test]
fn test_ffi_to_text() {
    let input = CString::new("Hello (world)[emphasis:\"strong\"]").unwrap();

    let result = ffi::speechmarkdown_to_text(input.as_ptr());
    assert!(!result.is_null());

    let output = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
    assert_eq!(output, "Hello world");

    ffi::speechmarkdown_free(result);
}

#[test]
fn test_ffi_parse() {
    let input = CString::new("Hello world").unwrap();

    let result = ffi::speechmarkdown_parse(input.as_ptr());
    assert!(!result.is_null());

    let output = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
    assert!(output.contains("\"node_type\""));
    assert!(output.contains("Hello world"));

    ffi::speechmarkdown_free(result);
}

#[test]
fn test_ffi_null_input() {
    let platform = CString::new("amazon-alexa").unwrap();

    let result = ffi::speechmarkdown_to_ssml(std::ptr::null(), platform.as_ptr());
    assert!(result.is_null());

    let error = ffi::speechmarkdown_get_error();
    assert!(!error.is_null());
    let error_msg = unsafe { CStr::from_ptr(error) }.to_str().unwrap();
    assert!(error_msg.contains("null"));
    ffi::speechmarkdown_free(error);
}

#[test]
fn test_ffi_invalid_platform() {
    let input = CString::new("Hello").unwrap();
    let platform = CString::new("invalid-platform").unwrap();

    let result = ffi::speechmarkdown_to_ssml(input.as_ptr(), platform.as_ptr());
    assert!(result.is_null());

    let error = ffi::speechmarkdown_get_error();
    assert!(!error.is_null());
    let error_msg = unsafe { CStr::from_ptr(error) }.to_str().unwrap();
    assert!(error_msg.contains("unsupported platform"));
    ffi::speechmarkdown_free(error);
}

#[test]
fn test_ffi_platform_aliases() {
    let aliases = vec![
        ("alexa", "amazon-alexa"),
        ("google", "google-assistant"),
        ("azure", "microsoft-azure"),
    ];

    for (alias, _) in aliases {
        let input = CString::new("Hello world").unwrap();
        let platform = CString::new(alias).unwrap();

        let result = ffi::speechmarkdown_to_ssml(input.as_ptr(), platform.as_ptr());
        assert!(
            !result.is_null(),
            "Platform alias '{}' should be valid",
            alias
        );
        ffi::speechmarkdown_free(result);
    }
}

#[test]
fn test_ffi_roundtrip_consistency() {
    let input = "Your balance is: (12345)[number;emphasis:\"strong\";whisper;pitch:\"high\"].";

    let rust_result = SpeechMarkdownParser::to_ssml(input, Platform::AmazonAlexa).unwrap();

    let c_input = CString::new(input).unwrap();
    let c_platform = CString::new("amazon-alexa").unwrap();
    let ffi_result_ptr = ffi::speechmarkdown_to_ssml(c_input.as_ptr(), c_platform.as_ptr());
    let ffi_result = unsafe { CStr::from_ptr(ffi_result_ptr) }
        .to_str()
        .unwrap()
        .to_string();
    ffi::speechmarkdown_free(ffi_result_ptr);

    assert_eq!(rust_result, ffi_result);
}
