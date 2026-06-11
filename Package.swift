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
            url: "https://github.com/AACTools/speechmarkdown-rust/releases/download/v0.4.4/SpeechMarkdownRust.xcframework.zip",
            checksum: "73dccb4d08f3b9d376f41cbe09c9c0881079116ac275b656f3081adf0b85f054"
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
