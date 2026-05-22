import SpeechMarkdown

let input = "Hello (world)[emphasis:\"strong\"]"

do {
    let parser = SpeechMarkdownParser()

    let ssml = try parser.toSsml(input: input, platform: "amazon-alexa")
    print("SSML: \(ssml)")

    let text = try parser.toText(input: input)
    print("Text: \(text)")

    let json = try parser.parseToJson(input: input)
    print("AST: \(json)")
} catch {
    print("Error: \(error)")
}
