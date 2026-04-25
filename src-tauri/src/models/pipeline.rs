use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    LocalFile,
    RemoteFile,
    PastedText,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

impl SourceSpan {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParseDiagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub span: Option<SourceSpan>,
}

impl ParseDiagnostic {
    pub fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        span: Option<SourceSpan>,
    ) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            code: code.into(),
            message: message.into(),
            span,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PipelineProperty {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PipelineNodeKind {
    Element,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PipelineNode {
    pub id: String,
    pub factory_name: String,
    pub instance_name: Option<String>,
    pub kind: PipelineNodeKind,
    pub properties: Vec<PipelineProperty>,
    pub source_span: SourceSpan,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PipelinePortKind {
    Src,
    Sink,
    Named,
    Request,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PipelinePort {
    pub id: String,
    pub node_id: String,
    pub port_kind: PipelinePortKind,
    pub port_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PipelineEdge {
    pub id: String,
    pub source_node_id: String,
    pub source_port: Option<PipelinePort>,
    pub target_node_id: String,
    pub target_port: Option<PipelinePort>,
    pub caps_label: Option<String>,
    pub source_span: SourceSpan,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PipelineGraph {
    pub nodes: Vec<PipelineNode>,
    pub edges: Vec<PipelineEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PipelineDocument {
    pub source_kind: SourceKind,
    pub path: Option<String>,
    pub source_name: Option<String>,
    pub raw_text: String,
    pub normalized_text: String,
    pub diagnostics: Vec<ParseDiagnostic>,
    pub graph: PipelineGraph,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizationResult {
    pub normalized_text: String,
    pub diagnostics: Vec<ParseDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoteTargetRequest {
    pub host: String,
    pub port: Option<u16>,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoteProbeResponse {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub version_output: String,
    pub sample_element_output: Option<String>,
}
