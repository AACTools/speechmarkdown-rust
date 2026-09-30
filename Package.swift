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
            url: "https://github.com/AACTools/speechmarkdown-rust/releases/download/v0.5.2/SpeechMarkdownRust.xcframework.zip",
            checksum: "a431afdf97e5069a9bb9d2109ede9db3aebc75298ee9e9bd742512f5e1fffb64"
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
