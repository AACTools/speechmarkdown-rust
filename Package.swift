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
            url: "https://github.com/AACTools/speechmarkdown-rust/releases/download/v0.4.3/SpeechMarkdownRust.xcframework.zip",
            checksum: "beb94ac3478890eda0cd63377123369e8ff23b41497dac93ecea8f656f7293b0"
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
