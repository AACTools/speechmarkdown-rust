const path = require('path');
const ffi = require('ffi-napi');
const ref = require('ref-napi');

const platform = process.platform;
let libPath;
if (platform === 'win32') {
    libPath = path.join(__dirname, '..', '..', 'target', 'release', 'speechmarkdown_rust');
} else if (platform === 'darwin') {
    libPath = path.join(__dirname, '..', '..', 'target', 'release', 'libspeechmarkdown_rust');
} else {
    libPath = path.join(__dirname, '..', '..', 'target', 'release', 'libspeechmarkdown_rust');
}

const stringPtr = ref.refType(ref.types.CString);

const lib = ffi.Library(libPath, {
    'speechmarkdown_to_ssml': ['string', ['string', 'string']],
    'speechmarkdown_to_text': ['string', ['string']],
    'speechmarkdown_parse': ['string', ['string']],
    'speechmarkdown_free': ['void', ['string']],
    'speechmarkdown_get_error': ['string', []],
});

class SpeechMarkdownParser {
    toSsml(input, platform) {
        const result = lib.speechmarkdown_to_ssml(input, platform);
        if (result === null || result === undefined) {
            const err = lib.speechmarkdown_get_error();
            throw new Error(err || 'Unknown error');
        }
        lib.speechmarkdown_free(result);
        return result;
    }

    toText(input) {
        const result = lib.speechmarkdown_to_text(input);
        if (result === null || result === undefined) {
            const err = lib.speechmarkdown_get_error();
            throw new Error(err || 'Unknown error');
        }
        lib.speechmarkdown_free(result);
        return result;
    }

    parseToJson(input) {
        const result = lib.speechmarkdown_parse(input);
        if (result === null || result === undefined) {
            const err = lib.speechmarkdown_get_error();
            throw new Error(err || 'Unknown error');
        }
        lib.speechmarkdown_free(result);
        return result;
    }
}

module.exports = { SpeechMarkdownParser };
