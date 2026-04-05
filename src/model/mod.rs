use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectorKind {
    Codex,
    ClaudeCode,
}

impl ConnectorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::ClaudeCode => "claude-code",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SupportStage {
    Current,
    Planned,
}

impl SupportStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Planned => "planned",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputFormat {
    Markdown,
    Json,
    Html,
}

impl OutputFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Json => "json",
            Self::Html => "html",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectorDefinition {
    pub kind: ConnectorKind,
    pub stage: SupportStage,
    pub summary: &'static str,
    pub source_of_truth: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectorKind {
    ThreadId(String),
    RolloutPath(PathBuf),
}
