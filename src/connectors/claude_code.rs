use crate::model::{ConnectorDefinition, ConnectorKind, SupportStage};

pub const DEFINITION: ConnectorDefinition = ConnectorDefinition {
    kind: ConnectorKind::ClaudeCode,
    stage: SupportStage::Planned,
    summary: "Planned follow-up connector after the Codex source adapter is stable.",
    source_of_truth: "External direct-read exporters and future agent-exporter connector interface",
};
