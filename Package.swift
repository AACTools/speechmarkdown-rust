// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "SpeechMarkdown",
    platforms: [.macOS(.v13), .iOS(.v16), .visionOS(.v1)],
    products: [
        .library(name: "SpeechMarkdown", targets: ["SpeechMarkdown"]),
    ],
    targets: [
        .binaryTarget(
            name: "SpeechMarkdownRust",
            url: "https://github.com/AACTools/speechmarkdown-rust/releases/download/v0.4.5/SpeechMarkdownRust.xcframework.zip",
            checksum: "dc8f01f9984e67e6bdfc6aa8efbe17bbace88f93ea5ecd6916decbf0678daada"
        ),
        .target(
            name: "CSpeechMarkdown",
            dependencies: ["SpeechMarkdownRust"],
            path: "bindings/swift/Sources/CSpeechMarkdown",
            publicHeadersPath: "include"
        ),
        .target(
            name: "SpeechMarkdown",
            dependencies: ["CSpeechMarkdown", "SpeechMarkdownRust"],
            path: "bindings/swift/Sources/SpeechMarkdown"
        ),
    ]
)
