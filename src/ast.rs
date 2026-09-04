use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Abstract Syntax Tree node for SpeechMarkdown
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AstNode {
    /// Type of the AST node
    pub node_type: NodeType,

    /// Text content of the node
    pub text: String,

    /// Child nodes
    pub children: Vec<AstNode>,

    /// Position in the source text (if available)
    pub position: Option<Position>,

    /// Additional attributes (modifier values, etc.)
    pub attributes: HashMap<String, String>,

    /// Ordered attribute keys (preserves insertion order)
    pub attribute_keys: Vec<String>,
}

impl AstNode {
    /// Create a new AST node
    pub fn new(node_type: NodeType, text: impl Into<String>) -> Self {
        Self {
            node_type,
            text: text.into(),
            children: Vec::new(),
            position: None,
            attributes: HashMap::new(),
            attribute_keys: Vec::new(),
        }
    }

    /// Add a child node
    pub fn add_child(mut self, child: AstNode) -> Self {
        self.children.push(child);
        self
    }

    /// Set position
    pub fn with_position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    /// Add an attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into();
        if !self.attributes.contains_key(&key) {
            self.attribute_keys.push(key.clone());
        }
        self.attributes.insert(key, value.into());
        self
    }

    /// Create a document node
    pub fn document() -> Self {
        Self::new(NodeType::Document, "")
    }

    /// Create a plain text node
    pub fn text(text: impl Into<String>) -> Self {
        Self::new(NodeType::PlainText, text)
    }
}

/// Position in source text
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
    /// Start byte offset
    pub start: usize,

    /// End byte offset
    pub end: usize,

    /// Line number (1-indexed)
    pub line: usize,

    /// Column number (1-indexed)
    pub column: usize,
}

impl Position {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }
}

/// Types of AST nodes in SpeechMarkdown
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    // Structural nodes
    /// Root document node
    Document,

    /// Paragraph node
    Paragraph,

    /// Simple line (no modifiers)
    SimpleLine,

    /// Empty line
    EmptyLine,

    /// Section with modifiers
    Section,

    // Content nodes
    /// Plain text content
    PlainText,

    /// Text with special characters
    PlainTextSpecialChars,

    /// Text in emphasis context
    PlainTextEmphasis,

    // Markup nodes
    /// Short break notation [time]
    ShortBreak,

    /// Extended break with strength
    Break,

    /// Moderate emphasis +text+
    ShortEmphasisModerate,

    /// Strong emphasis ++text++
    ShortEmphasisStrong,

    /// No emphasis ~text~
    ShortEmphasisNone,

    /// Reduced emphasis -text-
    ShortEmphasisReduced,

    /// Text modifier (text)[key:value]
    TextModifier,

    /// Short IPA notation (/text/phoneme/)
    ShortIpa,

    /// Bare IPA notation /phoneme/
    BareIpa,

    /// Short substitution {text}alias
    ShortSub,

    /// Audio element ![caption](url)
    Audio,

    /// Mark tag
    Mark,

    /// Expressive audio tag [laugh], [sigh], [applause], etc.
    /// Kept verbatim by formatters as a bracketed audio tag.
    Expressive,

    // Modifier types (for text modifiers and sections)
    /// Emphasis modifier
    Emphasis,

    /// Voice modifier
    Voice,

    /// Language modifier
    Lang,

    /// Rate modifier
    Rate,

    /// Pitch modifier
    Pitch,

    /// Volume modifier
    Volume,

    /// Whisper modifier
    Whisper,

    /// Excited modifier
    Excited,

    /// Disappointed modifier
    Disappointed,

    /// Newscaster modifier
    Newscaster,

    /// DJ modifier
    Dj,

    /// Date modifier
    Date,

    /// Time modifier
    Time,

    /// Number modifier
    Number,

    /// Ordinal modifier
    Ordinal,

    /// Characters modifier
    Characters,

    /// Fraction modifier
    Fraction,

    /// Telephone modifier
    Telephone,

    /// Unit modifier
    Unit,

    /// Address modifier
    Address,

    /// Interjection modifier
    Interjection,

    /// Expletive/Bleep modifier
    Expletive,

    /// IPA modifier
    Ipa,

    /// Substitution modifier
    Sub,
}

impl NodeType {
    /// Check if this node type represents emphasis
    pub fn is_emphasis(&self) -> bool {
        matches!(
            self,
            NodeType::ShortEmphasisModerate
                | NodeType::ShortEmphasisStrong
                | NodeType::ShortEmphasisNone
                | NodeType::ShortEmphasisReduced
                | NodeType::Emphasis
        )
    }

    /// Check if this node type represents a break/pause
    pub fn is_break(&self) -> bool {
        matches!(self, NodeType::ShortBreak | NodeType::Break)
    }

    /// Check if this node type represents a modifier
    pub fn is_modifier(&self) -> bool {
        matches!(
            self,
            NodeType::Emphasis
                | NodeType::Voice
                | NodeType::Lang
                | NodeType::Rate
                | NodeType::Pitch
                | NodeType::Volume
                | NodeType::Whisper
                | NodeType::Excited
                | NodeType::Disappointed
                | NodeType::Newscaster
                | NodeType::Dj
                | NodeType::Date
                | NodeType::Time
                | NodeType::Number
                | NodeType::Ordinal
                | NodeType::Characters
                | NodeType::Fraction
                | NodeType::Telephone
                | NodeType::Unit
                | NodeType::Address
                | NodeType::Interjection
                | NodeType::Expletive
                | NodeType::Ipa
                | NodeType::Sub
        )
    }
}
