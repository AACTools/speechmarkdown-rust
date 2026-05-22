use pyo3::prelude::*;
use speechmarkdown_rust::{Platform, SpeechMarkdownParser};

fn parse_platform(platform: &str) -> PyResult<Platform> {
    Platform::from_platform_str(platform).ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err(format!(
            "unsupported platform: '{}'. Use one of: amazon-alexa, google-assistant, microsoft-azure, apple, w3c, samsung-bixby, elevenlabs, ibm-watson",
            platform
        ))
    })
}

#[pyfunction]
fn to_ssml(input: &str, platform: &str) -> PyResult<String> {
    let p = parse_platform(platform)?;
    SpeechMarkdownParser::to_ssml(input, p)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn to_text(input: &str) -> PyResult<String> {
    SpeechMarkdownParser::to_text(input)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

#[pyfunction]
fn parse(input: &str) -> PyResult<String> {
    let ast = SpeechMarkdownParser::parse(input)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    serde_json::to_string(&ast)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

#[pymodule]
fn speechmarkdown(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(to_ssml, m)?)?;
    m.add_function(wrap_pyfunction!(to_text, m)?)?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    Ok(())
}
