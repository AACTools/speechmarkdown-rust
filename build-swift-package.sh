#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_DIR="$REPO_ROOT/swift-package-dist"

echo "Building Rust library for Apple Silicon..."
cargo build --release --target aarch64-apple-darwin --manifest-path "$REPO_ROOT/Cargo.toml"

echo "Building Rust library for Intel macOS..."
cargo build --release --target x86_64-apple-darwin --manifest-path "$REPO_ROOT/Cargo.toml"

echo "Creating universal static library..."
mkdir -p "$REPO_ROOT/target/universal"
lipo -create \
    "$REPO_ROOT/target/aarch64-apple-darwin/release/libspeechmarkdown_rust.a" \
    "$REPO_ROOT/target/x86_64-apple-darwin/release/libspeechmarkdown_rust.a" \
    -output "$REPO_ROOT/target/universal/libspeechmarkdown_rust.a"

echo "Preparing XCFramework..."
PREP_DIR=$(mktemp -d)
mkdir -p "$PREP_DIR/macos-archs"
cp "$REPO_ROOT/target/universal/libspeechmarkdown_rust.a" "$PREP_DIR/macos-archs/"
cp "$REPO_ROOT/bindings/speechmarkdown.h" "$PREP_DIR/macos-archs/"
cp "$REPO_ROOT/bindings/swift/Sources/CSpeechMarkdown/shim.h" "$PREP_DIR/macos-archs/"
cp "$REPO_ROOT/bindings/swift/Sources/CSpeechMarkdown/module.modulemap" "$PREP_DIR/macos-archs/"

rm -rf "$PREP_DIR/SpeechMarkdownRust.xcframework"
xcodebuild -create-xcframework \
    -library "$PREP_DIR/macos-archs/libspeechmarkdown_rust.a" \
    -headers "$PREP_DIR/macos-archs/" \
    -output "$PREP_DIR/SpeechMarkdownRust.xcframework"

echo "Assembling Swift package..."
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR/Sources/SpeechMarkdown"
mkdir -p "$DIST_DIR/Tests/SpeechMarkdownTests"

cp -R "$PREP_DIR/SpeechMarkdownRust.xcframework" "$DIST_DIR/"
cp "$REPO_ROOT/bindings/swift/SpeechMarkdown.swift" "$DIST_DIR/Sources/SpeechMarkdown/"

cat > "$DIST_DIR/Package.swift" << 'SWIFTPKG'
// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "SpeechMarkdown",
    platforms: [.macOS(.v12)],
    products: [
        .library(name: "SpeechMarkdown", targets: ["SpeechMarkdown"]),
    ],
    dependencies: [],
    targets: [
        .binaryTarget(
            name: "SpeechMarkdownRust",
            path: "SpeechMarkdownRust.xcframework"
        ),
        .target(
            name: "SpeechMarkdown",
            dependencies: ["SpeechMarkdownRust"],
            path: "Sources/SpeechMarkdown"
        ),
        .testTarget(
            name: "SpeechMarkdownTests",
            dependencies: ["SpeechMarkdown"],
            path: "Tests/SpeechMarkdownTests"
        ),
    ]
)
SWIFTPKG

cat > "$DIST_DIR/Tests/SpeechMarkdownTests/SpeechMarkdownTests.swift" << 'SWIFTTEST'
import XCTest
@testable import SpeechMarkdown

final class SpeechMarkdownTests: XCTestCase {

    let parser = SpeechMarkdownParser()

    func testIsSpeechMarkdown() {
        XCTAssertTrue(parser.isSpeechMarkdown(input: "Hello (world)[emphasis:\"strong\"]"))
        XCTAssertFalse(parser.isSpeechMarkdown(input: "Hello world"))
    }

    func testToSsml() throws {
        let ssml = try parser.toSsml(input: "Hello (world)[emphasis:\"strong\"]", platform: "amazon-alexa")
        XCTAssertTrue(ssml.contains("<emphasis"))
        XCTAssertTrue(ssml.contains("world"))
    }

    func testToText() throws {
        let text = try parser.toText(input: "Hello (world)[emphasis:\"strong\"]")
        XCTAssertEqual(text, "Hello world")
    }

    func testToSmd() throws {
        let smd = try parser.toSmd(ssml: "<speak><emphasis level=\"strong\">word</emphasis></speak>")
        XCTAssertEqual(smd, "++word++")
    }

    func testToSmdBreak() throws {
        let smd = try parser.toSmd(ssml: "<speak>Hello <break time=\"2s\"/> world</speak>")
        XCTAssertEqual(smd, "Hello [2s] world")
    }

    func testValidate() throws {
        try parser.validate(input: "Hello world")
    }
}
SWIFTTEST

echo "Verifying..."
cd "$DIST_DIR"
swift build
swift test

echo ""
echo "Swift package ready at: $DIST_DIR"
echo ""
echo "Usage:"
echo "  1. Copy swift-package-dist/ to your project"
echo "  2. In Xcode: File > Add Packages > Add Local > select the directory"
echo "  3. Or in Package.swift: .package(path: \"./path/to/swift-package-dist\")"
echo ""
echo "Programmatic usage:"
echo "  let parser = SpeechMarkdownParser()"
echo "  let ssml = try parser.toSsml(input: \"Hello ++world++\", platform: \"amazon-alexa\")"
echo "  let text = try parser.toText(input: \"Hello ++world++\")"
echo "  let smd  = try parser.toSmd(ssml: \"<speak><emphasis level=\\\"strong\\\">word</emphasis></speak>\")"
