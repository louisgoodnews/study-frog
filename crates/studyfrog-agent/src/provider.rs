//! Providers: where a model runs and how to talk to it.

/// The wire protocol of a provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// Ollama's `/api/chat` (local models).
    Ollama,
    /// OpenAI's `/v1/chat/completions`, which many servers copy.
    OpenAiCompatible,
    /// Anthropic's `/v1/messages`.
    Anthropic,
    /// Google's `generateContent` of the Generative Language API.
    Gemini,
    /// DeepSeek's `/chat/completions`.
    DeepSeek,
}

/// Every protocol, in the order the form offers them.
pub const PROTOCOLS: [Protocol; 5] = [
    Protocol::Ollama,
    Protocol::OpenAiCompatible,
    Protocol::Anthropic,
    Protocol::Gemini,
    Protocol::DeepSeek,
];

/// The word stored and shown for a protocol.
pub fn protocol_name(p: Protocol) -> &'static str {
    match p {
        Protocol::Ollama => "ollama",
        Protocol::OpenAiCompatible => "openai",
        Protocol::Anthropic => "anthropic",
        Protocol::Gemini => "gemini",
        Protocol::DeepSeek => "deepseek",
    }
}

/// The protocol with this stored word.
pub fn protocol_from_name(name: &str) -> Option<Protocol> {
    PROTOCOLS
        .into_iter()
        .find(|p| protocol_name(*p) == name.trim())
}

/// The address a protocol's usual server listens on.
pub fn default_base_url(p: Protocol) -> &'static str {
    match p {
        Protocol::Ollama => "http://localhost:11434",
        Protocol::OpenAiCompatible => "https://api.openai.com",
        Protocol::Anthropic => "https://api.anthropic.com",
        Protocol::Gemini => "https://generativelanguage.googleapis.com",
        Protocol::DeepSeek => "https://api.deepseek.com",
    }
}

/// The models a protocol suggests when a connection is added; the first is filled in. Empty when
/// the model is the user's own choice.
pub fn default_models(p: Protocol) -> &'static [&'static str] {
    match p {
        Protocol::Gemini => &["gemini-1.5-flash", "gemini-1.5-pro"],
        Protocol::DeepSeek => &["deepseek-chat", "deepseek-reasoner"],
        _ => &[],
    }
}

/// A model somewhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    /// Name the user gave it, unique.
    pub name: String,
    /// How to talk to it.
    pub protocol: Protocol,
    /// Server address without a trailing slash.
    pub base_url: String,
    /// The model to ask for.
    pub model: String,
    /// The user allowed text from the vault to be sent to this provider (only asked for remote ones).
    pub allowed: bool,
}

/// The host part of an address: `https://api.openai.com/v1` gives `api.openai.com`.
pub fn host_of(url: &str) -> &str {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let authority = rest.split('/').next().unwrap_or(rest);
    let authority = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    if authority.starts_with('[') {
        return authority.split(']').next().map_or(authority, |h| &h[1..]);
    }
    authority.split(':').next().unwrap_or(authority)
}

/// True when the provider runs on this computer, so no text leaves it.
pub fn is_local(p: &Provider) -> bool {
    matches!(host_of(&p.base_url), "localhost" | "127.0.0.1" | "::1")
}

/// True when the provider may be sent text: local ones always, remote ones once the user allowed it.
pub fn may_send(p: &Provider) -> bool {
    is_local(p) || p.allowed
}

/// Who said a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The user.
    User,
    /// The model.
    Assistant,
}

/// One turn of a conversation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// Who said it.
    pub role: Role,
    /// What was said.
    pub content: String,
}

/// A request ready to be sent (always a POST with a JSON body).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    /// Full address.
    pub url: String,
    /// Header names and values.
    pub headers: Vec<(String, String)>,
    /// The JSON body.
    pub body: String,
    /// Give up after this many seconds.
    pub timeout_secs: u32,
}

/// What came back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    /// Status code.
    pub status: u16,
    /// The body as text.
    pub body: String,
}

/// Sends a request; the application binary supplies it (an error is a sentence for the user).
pub type Transport = fn(&HttpRequest) -> Result<HttpResponse, String>;
