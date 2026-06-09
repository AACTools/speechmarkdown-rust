// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "SpeechMarkdown",
    platforms: [.macOS(.v13), .iOS(.v16)],
    products: [
        .library(name: "SpeechMarkdown", targets: ["SpeechMarkdown"]),
    ],
    targets: [
        .binaryTarget(
            name: "SpeechMarkdownRust",
            url: "https://github.com/AACTools/speechmarkdown-rust/releases/download/v0.2.7/SpeechMarkdownRust.xcframework.zip",
            checksum: "e91b0dba51b583e0ef61c17bff573ce0dd82a7557a7236dc70fcecb61485c4e2"
        ),
        .target(
            name: "CSpeechMarkdown",
            dependencies: ["SpeechMarkdownRust"],
            path: "bindings/swift/Sources/CSpeechMarkdown",
            publicHeadersPath: "include"
        ),
        .target(
            name: "SpeechMarkdown",
            dependencies: ["CSpeechMarkdown"],
            path: "bindings/swift/Sources/SpeechMarkdown",
            linkerSettings: [
                .linkedLibrary("speechmarkdown_rust")
            ]
        ),
    ]
)
