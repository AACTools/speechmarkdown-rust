use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::formatters::base::Platform;
use crate::parser::SpeechMarkdownParser;

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn set_last_error(msg: &str) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(
            CString::new(msg)
                .unwrap_or_else(|_| CString::new("error message contained null byte").unwrap()),
        );
    });
}

fn parse_platform(platform: *const c_char) -> Option<Platform> {
    if platform.is_null() {
        set_last_error("platform argument is null");
        return None;
    }
    let platform_str = unsafe { CStr::from_ptr(platform) }.to_str().unwrap_or("");
    match Platform::from_platform_str(platform_str) {
        Some(p) => Some(p),
        None => {
            set_last_error(&format!("unsupported platform: '{}'", platform_str));
            None
        }
    }
}

fn to_c_string(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(cs) => cs.into_raw(),
        Err(_) => {
            set_last_error("output contained null byte");
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// `input` and `platform` must be valid pointers to null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn speechmarkdown_to_ssml(
    input: *const c_char,
    platform: *const c_char,
) -> *mut c_char {
    if input.is_null() {
        set_last_error("input argument is null");
        return std::ptr::null_mut();
    }

    let input_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("input is not valid UTF-8");
            return std::ptr::null_mut();
        }
    };

    let platform_val = match parse_platform(platform) {
        Some(p) => p,
        None => return std::ptr::null_mut(),
    };

    match SpeechMarkdownParser::to_ssml(input_str, platform_val) {
        Ok(ssml) => to_c_string(ssml),
        Err(e) => {
            set_last_error(&e.to_string());
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// `input` must be a valid pointer to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn speechmarkdown_to_text(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        set_last_error("input argument is null");
        return std::ptr::null_mut();
    }

    let input_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("input is not valid UTF-8");
            return std::ptr::null_mut();
        }
    };

    match SpeechMarkdownParser::to_text(input_str) {
        Ok(text) => to_c_string(text),
        Err(e) => {
            set_last_error(&e.to_string());
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// `s` must be a valid pointer to a `CString` previously returned by this library, or null.
#[no_mangle]
pub unsafe extern "C" fn speechmarkdown_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

#[no_mangle]
pub extern "C" fn speechmarkdown_get_error() -> *mut c_char {
    LAST_ERROR.with(|e| match e.borrow().as_ref() {
        Some(cs) => {
            let copy = cs.clone();
            copy.into_raw()
        }
        None => std::ptr::null_mut(),
    })
}

/// # Safety
/// `input` must be a valid pointer to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn speechmarkdown_parse(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        set_last_error("input argument is null");
        return std::ptr::null_mut();
    }

    let input_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("input is not valid UTF-8");
            return std::ptr::null_mut();
        }
    };

    match SpeechMarkdownParser::parse(input_str) {
        Ok(ast) => match serde_json::to_string(&ast) {
            Ok(json) => to_c_string(json),
            Err(e) => {
                set_last_error(&format!("JSON serialization error: {}", e));
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            set_last_error(&e.to_string());
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// `input` must be a valid pointer to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn speechmarkdown_is_speech_markdown(input: *const c_char) -> bool {
    if input.is_null() {
        return false;
    }

    let input_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    SpeechMarkdownParser::is_speech_markdown(input_str)
}

/// # Safety
/// `input` must be a valid pointer to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn speechmarkdown_validate(input: *const c_char) -> bool {
    if input.is_null() {
        set_last_error("input argument is null");
        return false;
    }

    let input_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("input is not valid UTF-8");
            return false;
        }
    };

    match SpeechMarkdownParser::validate(input_str) {
        Ok(()) => true,
        Err(e) => {
            set_last_error(&e.to_string());
            false
        }
    }
}

/// # Safety
/// `input` must be a valid pointer to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn speechmarkdown_to_smd(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        set_last_error("input argument is null");
        return std::ptr::null_mut();
    }

    let input_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("input is not valid UTF-8");
            return std::ptr::null_mut();
        }
    };

    match SpeechMarkdownParser::to_smd(input_str) {
        Ok(smd) => to_c_string(smd),
        Err(e) => {
            set_last_error(&e.to_string());
            std::ptr::null_mut()
        }
    }
}
