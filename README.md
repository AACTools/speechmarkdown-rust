# SpeechMarkdown Rust

High-performance SpeechMarkdown parser written in Rust. Converts [SpeechMarkdown](https://speechmarkdown.com/) syntax to platform-specific SSML for Amazon Alexa, Google Assistant, Microsoft Azure, and more.

## Usage

### Rust

```rust
use speechmarkdown_rust::{SpeechMarkdownParser, Platform};

// Convert to SSML
let ssml = SpeechMarkdownParser::to_ssml(
    "Hello (world)[emphasis:\"strong\"]",
    Platform::AmazonAlexa,
)?;
// => <speak>Hello <emphasis level="strong">world</emphasis></speak>

// Convert to plain text
let text = SpeechMarkdownParser::to_text("Hello (world)[emphasis:\"strong\"]")?;
// => Hello world

// Parse to AST (JSON)
let ast = SpeechMarkdownParser::parse("Hello world")?;
```

### Supported Platforms

| Platform | Enum Value | String ID |
|----------|-----------|-----------|
| Amazon Alexa | `Platform::AmazonAlexa` | `"amazon-alexa"` or `"alexa"` |
| Google Assistant | `Platform::GoogleAssistant` | `"google-assistant"` or `"google"` |
| Microsoft Azure | `Platform::MicrosoftAzure` | `"microsoft-azure"` or `"azure"` |
| Apple | `Platform::Apple` | `"apple"` |
| W3C | `Platform::W3c` | `"w3c"` |
| Samsung Bixby | `Platform::SamsungBixby` | `"samsung-bixby"` or `"bixby"` |
| ElevenLabs | `Platform::ElevenLabs` | `"elevenlabs"` |
| IBM Watson | `Platform::IbmWatson` | `"ibm-watson"` or `"watson"` |

## Language Bindings

The library exposes a C ABI (`cdylib`/`staticlib`) so it can be used from any language. Pre-built bindings are provided for .NET, Swift, and Node.js.

### Build the native library

```bash
cargo build --release
```

This produces:
- **Windows**: `target/release/speechmarkdown_rust.dll`
- **macOS**: `target/release/libspeechmarkdown_rust.dylib`
- **Linux**: `target/release/libspeechmarkdown_rust.so`

### C API

```c
#include "bindings/speechmarkdown.h"

// Convert SpeechMarkdown to SSML
const char* ssml = speechmarkdown_to_ssml("Hello (world)[emphasis:\"strong\"]", "amazon-alexa");
printf("%s\n", ssml);
speechmarkdown_free((char*)ssml);

// Convert to plain text
const char* text = speechmarkdown_to_text("Hello (world)[emphasis:\"strong\"]");
printf("%s\n", text);
speechmarkdown_free((char*)text);

// Get last error (thread-local)
const char* err = speechmarkdown_get_error();
```

### .NET (C#)

```csharp
using SpeechMarkdown;

var parser = new SpeechMarkdownParser();

string ssml = parser.ToSsml("Hello (world)[emphasis:\"strong\"]", Platform.AmazonAlexa);
string text = parser.ToText("Hello (world)[emphasis:\"strong\"]");
string json = parser.ParseToJson("Hello world");
```

Copy `bindings/dotnet/SpeechMarkdown.cs` into your project and place the native library in your output directory alongside the assembly.

### Swift

```swift
import SpeechMarkdown

let parser = SpeechMarkdownParser()

let ssml = try parser.toSsml(input: "Hello (world)[emphasis:\"strong\"]", platform: "amazon-alexa")
let text = try parser.toText(input: "Hello (world)[emphasis:\"strong\"]")
let json = try parser.parseToJson(input: "Hello world")
```

Include `bindings/swift/SpeechMarkdown.swift`, `bindings/swift/module.modulemap`, and `bindings/speechmarkdown.h` in your Xcode project. Link against the compiled `.dylib`.

### Node.js

```js
const { SpeechMarkdownParser } = require('./bindings/nodejs');

const parser = new SpeechMarkdownParser();

const ssml = parser.toSsml('Hello (world)[emphasis:"strong"]', 'amazon-alexa');
const text = parser.toText('Hello (world)[emphasis:"strong"]');
const json = parser.parseToJson('Hello world');
```

Install `ffi-napi` and `ref-napi` as dependencies. Build the native library first with `cargo build --release`.

### Python

```python
from speechmarkdown import to_ssml, to_text, parse

ssml = to_ssml('Hello (world)[emphasis:"strong"]', 'amazon-alexa')
text = to_text('Hello (world)[emphasis:"strong"]')
ast = parse('Hello world')  # returns dict
```

Available on [PyPI](https://pypi.org/project/speechmarkdown-rust/) as `speechmarkdown-rust`.
