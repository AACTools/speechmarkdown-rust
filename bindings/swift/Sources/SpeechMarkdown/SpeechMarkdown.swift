import Foundation
import CSpeechMarkdown

public class SpeechMarkdownParser {

    private static let lock = NSLock()

    public init() {}

    public func isSpeechMarkdown(input: String) -> Bool {
        return input.withCString { inputPtr in
            speechmarkdown_is_speech_markdown(inputPtr)
        }
    }

    public func validate(input: String) throws {
        let valid = input.withCString { inputPtr in
            speechmarkdown_validate(inputPtr)
        }
        if !valid {
            throw SpeechMarkdownError.fromLastError(context: "validate")
        }
    }

    public func toSsml(input: String, platform: String) throws -> String {
        return try SpeechMarkdownParser.lock.withLock {
            let result: String? = try input.withCString { inputPtr in
                try platform.withCString { platformPtr in
                    let ptr = speechmarkdown_to_ssml(inputPtr, platformPtr)
                    if ptr == nil {
                        throw SpeechMarkdownError.fromLastError(context: "toSsml")
                    }
                    let str = String(cString: ptr!)
                    speechmarkdown_free(UnsafeMutablePointer(mutating: ptr!))
                    return str
                }
            }
            return result!
        }
    }

    public func toText(input: String) throws -> String {
        return try SpeechMarkdownParser.lock.withLock {
            let result: String? = try input.withCString { inputPtr in
                let ptr = speechmarkdown_to_text(inputPtr)
                if ptr == nil {
                    throw SpeechMarkdownError.fromLastError(context: "toText")
                }
                let str = String(cString: ptr!)
                speechmarkdown_free(UnsafeMutablePointer(mutating: ptr!))
                return str
            }
            return result!
        }
    }

    public func parseToJson(input: String) throws -> String {
        return try SpeechMarkdownParser.lock.withLock {
            let result: String? = try input.withCString { inputPtr in
                let ptr = speechmarkdown_parse(inputPtr)
                if ptr == nil {
                    throw SpeechMarkdownError.fromLastError(context: "parseToJson")
                }
                let str = String(cString: ptr!)
                speechmarkdown_free(UnsafeMutablePointer(mutating: ptr!))
                return str
            }
            return result!
        }
    }

    public func toSmd(ssml: String) throws -> String {
        return try SpeechMarkdownParser.lock.withLock {
            let result: String? = try ssml.withCString { inputPtr in
                let ptr = speechmarkdown_to_smd(inputPtr)
                if ptr == nil {
                    throw SpeechMarkdownError.fromLastError(context: "toSmd")
                }
                let str = String(cString: ptr!)
                speechmarkdown_free(UnsafeMutablePointer(mutating: ptr!))
                return str
            }
            return result!
        }
    }

    public func supportedSsml(platform: String) throws -> String {
        return try SpeechMarkdownParser.lock.withLock {
            let result: String? = try platform.withCString { platformPtr in
                let ptr = speechmarkdown_supported_ssml(platformPtr)
                if ptr == nil {
                    throw SpeechMarkdownError.fromLastError(context: "supportedSsml")
                }
                let str = String(cString: ptr!)
                speechmarkdown_free(UnsafeMutablePointer(mutating: ptr!))
                return str
            }
            return result!
        }
    }
}

public enum SpeechMarkdownError: Error, LocalizedError {
    case error(String)

    public var errorDescription: String? {
        switch self {
        case .error(let msg):
            return msg
        }
    }

    static func fromLastError(context: String) -> SpeechMarkdownError {
        guard let errorPtr = speechmarkdown_get_error() else {
            return .error("\(context): unknown error")
        }
        let msg = String(cString: errorPtr)
        speechmarkdown_free(UnsafeMutablePointer(mutating: errorPtr))
        return .error("\(context): \(msg)")
    }
}
