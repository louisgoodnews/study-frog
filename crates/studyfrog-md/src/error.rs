//! Typed errors. `Display` delegates to a free function (P1).

use std::fmt;

/// What went wrong while parsing a file.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    /// The file does not start with a `---` fenced block (or never closes it).
    MissingFrontMatter,
    /// The YAML could not be decoded; carries the decoder's message.
    InvalidYaml(String),
    /// A required key is absent.
    MissingKey(&'static str),
    /// A key has an unusable value.
    InvalidValue {
        /// Offending key.
        key: String,
        /// Why it was rejected.
        reason: String,
    },
    /// `kind` is not one of stack, note, card, question.
    UnknownKind(String),
    /// A card or question body has no `???` separator line.
    MissingSeparator,
}

/// A parse failure with the 1-based file line when known. The scanner adds the file path.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// Line in the file, when it can be located.
    pub line: Option<usize>,
    /// The failure.
    pub kind: ParseErrorKind,
}

/// What went wrong while rendering a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    /// A timestamp is outside years 0000 to 9999.
    TimestampOutOfRange,
    /// The front text would not split back at the separator (for example an unclosed code fence).
    AmbiguousSeparator,
    /// The YAML encoder failed; carries its message.
    Yaml(String),
}

/// Human-readable description of a parse error.
pub fn describe_parse_error(e: &ParseError) -> String {
    let at = e.line.map(|l| format!("line {l}: ")).unwrap_or_default();
    let what = match &e.kind {
        ParseErrorKind::MissingFrontMatter => "missing `---` front-matter block".to_owned(),
        ParseErrorKind::InvalidYaml(m) => format!("invalid YAML: {m}"),
        ParseErrorKind::MissingKey(k) => format!("missing required key `{k}`"),
        ParseErrorKind::InvalidValue { key, reason } => format!("invalid `{key}`: {reason}"),
        ParseErrorKind::UnknownKind(k) => format!("unknown kind `{k}`"),
        ParseErrorKind::MissingSeparator => "missing `???` separator line".to_owned(),
    };
    format!("{at}{what}")
}

/// Human-readable description of a render error.
pub fn describe_render_error(e: &RenderError) -> String {
    match e {
        RenderError::TimestampOutOfRange => "timestamp outside years 0000-9999".to_owned(),
        RenderError::AmbiguousSeparator => {
            "front text would swallow the `???` separator".to_owned()
        }
        RenderError::Yaml(m) => format!("YAML encoding failed: {m}"),
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&describe_parse_error(self))
    }
}

impl std::error::Error for ParseError {}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&describe_render_error(self))
    }
}

impl std::error::Error for RenderError {}
