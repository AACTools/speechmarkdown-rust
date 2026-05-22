#ifndef SPEECHMARKDOWN_H
#define SPEECHMARKDOWN_H

#include <stdlib.h>

#ifdef __cplusplus
extern "C" {
#endif

// Convert SpeechMarkdown input to SSML for the given platform.
// Platforms: "amazon-alexa", "google-assistant", "microsoft-azure",
//            "apple", "w3c", "samsung-bixby", "eleven-labs", "ibm-watson"
// Returns: allocated string with SSML, or NULL on error.
//          Caller must free with speechmarkdown_free().
const char* speechmarkdown_to_ssml(const char* input, const char* platform);

// Convert SpeechMarkdown input to plain text (strips all markup).
// Returns: allocated string, or NULL on error.
//          Caller must free with speechmarkdown_free().
const char* speechmarkdown_to_text(const char* input);

// Parse SpeechMarkdown input and return AST as JSON string.
// Returns: allocated JSON string, or NULL on error.
//          Caller must free with speechmarkdown_free().
const char* speechmarkdown_parse(const char* input);

// Free a string previously returned by any speechmarkdown_* function.
void speechmarkdown_free(char* s);

// Get the last error message (thread-local).
// Returns: allocated string with error message, or NULL if no error.
//          Caller must free with speechmarkdown_free().
const char* speechmarkdown_get_error(void);

bool speechmarkdown_is_speech_markdown(const char* input);

bool speechmarkdown_validate(const char* input);

#ifdef __cplusplus
}
#endif

#endif // SPEECHMARKDOWN_H
