use napi::bindgen_prelude::*;
use napi_derive::napi;
use speechmarkdown_rust::{Platform, SpeechMarkdownParser};

fn parse_platform(platform: &str) -> Result<Platform> {
    Platform::from_platform_str(platform).ok_or_else(|| {
        Error::from_reason(format!(
            "unsupported platform: '{}'. Use one of: amazon-alexa, google-assistant, microsoft-azure, apple, w3c, samsung-bixby, elevenlabs, ibm-watson",
            platform
        ))
    })
}

#[napi]
pub fn to_ssml(input: String, platform: String) -> Result<String> {
    let p = parse_platform(&platform)?;
    SpeechMarkdownParser::to_ssml(&input, p).map_err(|e| Error::from_reason(e.to_string()))
}

#[napi]
pub fn to_text(input: String) -> Result<String> {
    SpeechMarkdownParser::to_text(&input).map_err(|e| Error::from_reason(e.to_string()))
}

#[napi]
pub fn parse(input: String) -> Result<String> {
    let ast = SpeechMarkdownParser::parse(&input).map_err(|e| Error::from_reason(e.to_string()))?;
    serde_json::to_string(&ast).map_err(|e| Error::from_reason(e.to_string()))
}
