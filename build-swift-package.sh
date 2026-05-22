#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$SCRIPT_DIR"
DIST_DIR="$REPO_ROOT/swift-package-dist"

echo "Installing iOS targets..."
rustup target add aarch64-apple-ios aarch64-apple-ios-sim 2>/dev/null || true

echo "Building Rust library for macOS (Apple Silicon)..."
cargo build --release --target aarch64-apple-darwin --manifest-path "$REPO_ROOT/Cargo.toml"

echo "Building Rust library for macOS (Intel)..."
cargo build --release --target x86_64-apple-darwin --manifest-path "$REPO_ROOT/Cargo.toml"

echo "Building Rust library for iOS device..."
cargo build --release --target aarch64-apple-ios --manifest-path "$REPO_ROOT/Cargo.toml"

echo "Building Rust library for iOS simulator..."
cargo build --release --target aarch64-apple-ios-sim --manifest-path "$REPO_ROOT/Cargo.toml"

echo "Preparing XCFramework..."
PREP_DIR=$(mktemp -d)

# macOS universal (arm64 + x86_64)
mkdir -p "$PREP_DIR/macos-arm64_x86_64"
lipo -create \
    "$REPO_ROOT/target/aarch64-apple-darwin/release/libspeechmarkdown_rust.a" \
    "$REPO_ROOT/target/x86_64-apple-darwin/release/libspeechmarkdown_rust.a" \
    -output "$PREP_DIR/macos-arm64_x86_64/libspeechmarkdown_rust.a"
strip -x "$PREP_DIR/macos-arm64_x86_64/libspeechmarkdown_rust.a"
cp "$REPO_ROOT/bindings/speechmarkdown.h" "$PREP_DIR/macos-arm64_x86_64/"
cp "$REPO_ROOT/bindings/swift/Sources/CSpeechMarkdown/shim.h" "$PREP_DIR/macos-arm64_x86_64/"
cp "$REPO_ROOT/bindings/swift/Sources/CSpeechMarkdown/module.modulemap" "$PREP_DIR/macos-arm64_x86_64/"

# iOS device (arm64)
mkdir -p "$PREP_DIR/ios-arm64"
cp "$REPO_ROOT/target/aarch64-apple-ios/release/libspeechmarkdown_rust.a" "$PREP_DIR/ios-arm64/"
strip -x "$PREP_DIR/ios-arm64/libspeechmarkdown_rust.a"
cp "$REPO_ROOT/bindings/speechmarkdown.h" "$PREP_DIR/ios-arm64/"
cp "$REPO_ROOT/bindings/swift/Sources/CSpeechMarkdown/shim.h" "$PREP_DIR/ios-arm64/"
cp "$REPO_ROOT/bindings/swift/Sources/CSpeechMarkdown/module.modulemap" "$PREP_DIR/ios-arm64/"

# iOS simulator (arm64)
mkdir -p "$PREP_DIR/ios-arm64-sim"
cp "$REPO_ROOT/target/aarch64-apple-ios-sim/release/libspeechmarkdown_rust.a" "$PREP_DIR/ios-arm64-sim/"
strip -x "$PREP_DIR/ios-arm64-sim/libspeechmarkdown_rust.a"
cp "$REPO_ROOT/bindings/speechmarkdown.h" "$PREP_DIR/ios-arm64-sim/"
cp "$REPO_ROOT/bindings/swift/Sources/CSpeechMarkdown/shim.h" "$PREP_DIR/ios-arm64-sim/"
cp "$REPO_ROOT/bindings/swift/Sources/CSpeechMarkdown/module.modulemap" "$PREP_DIR/ios-arm64-sim/"

rm -rf "$PREP_DIR/SpeechMarkdownRust.xcframework"
xcodebuild -create-xcframework \
    -library "$PREP_DIR/macos-arm64_x86_64/libspeechmarkdown_rust.a" -headers "$PREP_DIR/macos-arm64_x86_64/" \
    -library "$PREP_DIR/ios-arm64/libspeechmarkdown_rust.a" -headers "$PREP_DIR/ios-arm64/" \
    -library "$PREP_DIR/ios-arm64-sim/libspeechmarkdown_rust.a" -headers "$PREP_DIR/ios-arm64-sim/" \
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
    platforms: [.macOS(.v13), .iOS(.v16)],
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
    }

    func testToText() throws {
        let text = try parser.toText(input: "Hello (world)[emphasis:\"strong\"]")
        XCTAssertEqual(text, "Hello world")
    }

    func testToSmd() throws {
        let smd = try parser.toSmd(ssml: "<speak><emphasis level=\"strong\">word</emphasis></speak>")
        XCTAssertEqual(smd, "++word++")
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
echo "Platforms: macOS (arm64 + x86_64), iOS device (arm64), iOS simulator (arm64)"
echo ""
echo "Usage:"
echo "  1. Copy swift-package-dist/ to your project"
echo "  2. In Xcode: File > Add Packages > Add Local > select the directory"
echo "  3. Or in Package.swift: .package(path: \"./path/to/swift-package-dist\")"
