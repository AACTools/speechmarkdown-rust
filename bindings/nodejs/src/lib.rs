use napi::bindgen_prelude::*;
use napi_derive::napi;
use speechmarkdown_rust::{Platform, SpeechMarkdownParser};

fn parse_platform(platform: &str) -> Result<Platform> {
    Platform::from_platform_str(platform).ok_or_else(|| {
        Error::from_reason(format!(
            "unsupported platform: '{}'. Use one of: amazon-alexa, google-assistant, microsoft-azure, apple, w3c, samsung-bixby, elevenlabs, elevenlabs-v3, ibm-watson",
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

#[napi]
pub fn is_speech_markdown(input: String) -> Result<bool> {
    Ok(SpeechMarkdownParser::is_speech_markdown(&input))
}

#[napi]
pub fn validate(input: String) -> Result<bool> {
    match SpeechMarkdownParser::validate(&input) {
        Ok(()) => Ok(true),
        Err(e) => Err(Error::from_reason(e.to_string())),
    }
}

#[napi]
pub fn to_smd(ssml: String) -> Result<String> {
    SpeechMarkdownParser::to_smd(&ssml).map_err(|e| Error::from_reason(e.to_string()))
}

#[napi]
pub fn supported_ssml(platform: String) -> Result<String> {
    let p = parse_platform(&platform)?;
    let caps = SpeechMarkdownParser::supported_ssml(p);
    serde_json::to_string(&caps).map_err(|e| Error::from_reason(e.to_string()))
}
