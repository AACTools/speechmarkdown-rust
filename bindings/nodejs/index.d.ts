export class SpeechMarkdownParser {
    toSsml(input: string, platform: string): string;
    toText(input: string): string;
    parseToJson(input: string): string;
}

export const Platform = {
    AmazonAlexa: 'amazon-alexa',
    GoogleAssistant: 'google-assistant',
    MicrosoftAzure: 'microsoft-azure',
    Apple: 'apple',
    W3c: 'w3c',
    SamsungBixby: 'samsung-bixby',
    ElevenLabs: 'eleven-labs',
    IbmWatson: 'ibm-watson',
} as const;

export type PlatformType = typeof Platform[keyof typeof Platform];
