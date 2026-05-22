def to_ssml(input: str, platform: str) -> str:
    """Convert SpeechMarkdown input to SSML for the given platform.

    Args:
        input: SpeechMarkdown text
        platform: One of 'amazon-alexa', 'google-assistant', 'microsoft-azure',
                  'apple', 'w3c', 'samsung-bixby', 'elevenlabs', 'ibm-watson'
    Returns:
        SSML string
    """
    ...

def to_text(input: str) -> str:
    """Convert SpeechMarkdown input to plain text (strips all markup).

    Args:
        input: SpeechMarkdown text
    Returns:
        Plain text string
    """
    ...

def parse(input: str) -> str:
    """Parse SpeechMarkdown input and return AST as JSON string.

    Args:
        input: SpeechMarkdown text
    Returns:
        JSON string representing the AST
    """
    ...
