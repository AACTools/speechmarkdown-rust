# SpeechMarkdown Rust

High-performance SpeechMarkdown parser written in Rust. Converts [SpeechMarkdown](https://speechmarkdown.com/) syntax to platform-specific SSML for Amazon Alexa, Google Assistant, Microsoft Azure, and more.

## Install

| Language | Package | Install |
|----------|---------|---------|
| Rust | [speechmarkdown-rust](https://crates.io/crates/speechmarkdown-rust) | `cargo add speechmarkdown-rust` |
| Python | [speechmarkdown-rust](https://pypi.org/project/speechmarkdown-rust/) | `pip install speechmarkdown-rust` |
| Node.js | [speechmarkdown](https://www.npmjs.com/package/speechmarkdown) | `npm install speechmarkdown` |
| .NET | [SpeechMarkdown](https://www.nuget.org/packages/SpeechMarkdown) | `dotnet add package SpeechMarkdown` |
| Swift | [Release asset](https://github.com/AACTools/speechmarkdown-rust/releases) | See Swift section below |

## Supported Platforms

| Platform | String ID |
|----------|-----------|
| Amazon Alexa | `"amazon-alexa"` or `"alexa"` |
| Google Assistant | `"google-assistant"` or `"google"` |
| Microsoft Azure | `"microsoft-azure"` or `"azure"` |
| Apple | `"apple"` |
| W3C | `"w3c"` |
| Samsung Bixby | `"samsung-bixby"` or `"bixby"` |
| ElevenLabs | `"elevenlabs"` |
| IBM Watson | `"ibm-watson"` or `"watson"` |

## API

All bindings expose the same core methods:

| Method | Returns | Description |
|--------|---------|-------------|
| `to_ssml(input, platform)` | `string` | Convert SpeechMarkdown to SSML for the given platform |
| `to_text(input)` | `string` | Convert SpeechMarkdown to plain text (strips all markup) |
| `to_smd(ssml)` | `string` | Convert SSML to SpeechMarkdown (best-effort, lossy for unsupported elements) |
| `parse(input)` | `string` (JSON) | Parse SpeechMarkdown and return the AST as JSON |
| `is_speech_markdown(input)` | `bool` | Check if a string contains SpeechMarkdown syntax |
| `validate(input)` | `bool` | Validate that SpeechMarkdown parses without errors |

## Usage

### Rust

```rust
use speechmarkdown_rust::{SpeechMarkdownParser, Platform};

// Convert to SSML
let ssml = SpeechMarkdownParser::to_ssml(
    "Hello (world)[emphasis:\"strong\"]",
    Platform::AmazonAlexa,
)?;

// Convert to plain text
let text = SpeechMarkdownParser::to_text("Hello (world)[emphasis:\"strong\"]")?;

// Parse to AST (JSON string)
let ast = SpeechMarkdownParser::parse("Hello world")?;

// Check if input contains SpeechMarkdown syntax
if SpeechMarkdownParser::is_speech_markdown(&input) {
    // ...
}

// Validate input
SpeechMarkdownParser::validate(&input)?;

// Convert SSML back to SpeechMarkdown (best-effort)
let smd = SpeechMarkdownParser::to_smd(r#"<speak><emphasis level="strong">word</emphasis></speak>"#)?;
// Returns: ++word++
```

### Python

```python
from speechmarkdown_rust import to_ssml, to_text, parse, is_speech_markdown, validate

ssml = to_ssml('Hello (world)[emphasis:"strong"]', 'amazon-alexa')
text = to_text('Hello (world)[emphasis:"strong"]')
ast = parse('Hello world')

is_smd = is_speech_markdown('Hello (world)[emphasis:"strong"]')  # True
is_smd = is_speech_markdown('Hello world')                       # False

validate('Hello (world)[emphasis:"strong"]')  # raises ValueError if invalid

# Convert SSML to SpeechMarkdown (best-effort)
smd = to_smd('<speak><emphasis level="strong">word</emphasis></speak>')
# Returns: ++word++
```

### Node.js

```js
const { to_ssml, to_text, parse, is_speech_markdown, validate } = require('speechmarkdown')

const ssml = to_ssml('Hello (world)[emphasis:"strong"]', 'amazon-alexa')
const text = to_text('Hello (world)[emphasis:"strong"]')
const ast = parse('Hello world')

is_speech_markdown('Hello (world)[emphasis:"strong"]')  // true
is_speech_markdown('Hello world')                       // false

validate('Hello (world)[emphasis:"strong"]')  // throws if invalid

// Convert SSML to SpeechMarkdown (best-effort)
const smd = to_smd('<speak><emphasis level="strong">word</emphasis></speak>')
// Returns: ++word++
```

### .NET (C#)

```csharp
using SpeechMarkdown;

var parser = new SpeechMarkdownParser();

string ssml = parser.ToSsml("Hello (world)[emphasis:\"strong\"]", Platform.AmazonAlexa);
string text = parser.ToText("Hello (world)[emphasis:\"strong\"]");
string json = parser.ParseToJson("Hello world");

bool isSmd = parser.IsSpeechMarkdown("Hello (world)[emphasis:\"strong\"]"); // true
parser.Validate("Hello (world)[emphasis:\"strong\"]"); // throws on invalid

// Convert SSML to SpeechMarkdown (best-effort)
string smd = parser.ToSmd("<speak><emphasis level=\"strong\">word</emphasis></speak>");
// Returns: ++word++
```

### Swift

The repo root contains a `Package.swift` with a binary target pointing to a pre-built XCFramework. This enables both local development and [Swift Package Index](https://swiftpackageindex.com/) integration.

**Option 1 — SPM (recommended):**
```swift
// In your Package.swift:
.package(url: "https://github.com/AACTools/speechmarkdown-rust", branch: "spm")
```

**Option 2 — Local package:**
Download `speechmarkdown-swift-package.zip` from the [latest release](https://github.com/AACTools/speechmarkdown-rust/releases), unzip, and add as a local package in Xcode (File > Add Packages > Add Local).

Includes macOS (arm64 + x86_64), iOS device (arm64), and iOS simulator (arm64) slices.

To build from source: `./build-swift-package.sh`

```swift
import SpeechMarkdown

let parser = SpeechMarkdownParser()

let ssml = try parser.toSsml(input: "Hello (world)[emphasis:\"strong\"]", platform: "amazon-alexa")
let text = try parser.toText(input: "Hello (world)[emphasis:\"strong\"]")
let json = try parser.parseToJson(input: "Hello world")

let isSmd = parser.isSpeechMarkdown(input: "Hello (world)[emphasis:\"strong\"]") // true
try parser.validate(input: "Hello (world)[emphasis:\"strong\"]") // throws on invalid

// Convert SSML to SpeechMarkdown (best-effort)
let smd = try parser.toSmd(ssml: "<speak><emphasis level=\"strong\">word</emphasis></speak>")
// Returns: ++word++
```

### C API

```c
#include "speechmarkdown.h"

// Convert to SSML
const char* ssml = speechmarkdown_to_ssml("Hello (world)[emphasis:\"strong\"]", "amazon-alexa");
speechmarkdown_free((char*)ssml);

// Convert to plain text
const char* text = speechmarkdown_to_text("Hello (world)[emphasis:\"strong\"]");
speechmarkdown_free((char*)text);

// Parse to JSON
const char* json = speechmarkdown_parse("Hello world");
speechmarkdown_free((char*)json);

// Check for SpeechMarkdown syntax
bool is_smd = speechmarkdown_is_speech_markdown("Hello (world)[emphasis:\"strong\"]");

// Validate
bool valid = speechmarkdown_validate("Hello (world)[emphasis:\"strong\"]");

// Convert SSML to SpeechMarkdown (best-effort)
const char* smd = speechmarkdown_to_smd("<speak><emphasis level=\"strong\">word</emphasis></speak>");
speechmarkdown_free((char*)smd);

// Get last error (thread-local)
const char* err = speechmarkdown_get_error();
```

## Building from Source

```bash
cargo build --release
```

This produces:
- **Windows**: `target/release/speechmarkdown_rust.dll`
- **macOS**: `target/release/libspeechmarkdown_rust.dylib`
- **Linux**: `target/release/libspeechmarkdown_rust.so`

## License

MIT
