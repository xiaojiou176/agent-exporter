mod claude_code;
mod codex;

use anyhow::{Result, bail};

use crate::core::archive::{ArchiveTranscript, ExportRequest};
use crate::model::{ConnectorDefinition, ConnectorKind};

pub fn catalog() -> &'static [ConnectorDefinition] {
    &[codex::DEFINITION, claude_code::DEFINITION]
}

pub fn find(kind: ConnectorKind) -> &'static ConnectorDefinition {
    match kind {
        ConnectorKind::Codex => &codex::DEFINITION,
        ConnectorKind::ClaudeCode => &claude_code::DEFINITION,
    }
}

pub fn export(request: &ExportRequest) -> Result<ArchiveTranscript> {
    match request.connector {
        ConnectorKind::Codex => codex::load_transcript(request),
        ConnectorKind::ClaudeCode => bail!(
            "claude-code remains planned only; Codex-only v1 ships the canonical Codex export path first"
        ),
    }
}
