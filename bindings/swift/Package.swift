// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "SpeechMarkdown",
    products: [
        .library(name: "SpeechMarkdown", targets: ["SpeechMarkdown"]),
    ],
    targets: [
        .systemLibrary(
            name: "CSpeechMarkdown",
            path: "Sources/CSpeechMarkdown"
        ),
        .target(
            name: "SpeechMarkdown",
            dependencies: ["CSpeechMarkdown"],
            path: "Sources/SpeechMarkdown"
        ),
    ]
)
