type PipelineSourceKind = 'local_file' | 'remote_file' | 'pasted_text' | 'sample'
type PipelineNodeKind = 'element' | 'virtual_group' | 'caps' | 'unknown'
type PipelineNodeTone =
  | 'source'
  | 'branch'
  | 'merge'
  | 'ai'
  | 'output'
  | 'utility'
  | 'unknown'
type DiagnosticSeverity = 'info' | 'warning' | 'error'

type SourceSpan = {
  lineStart: number
  lineEnd: number
}

type PipelineProperty = {
  key: string
  value: string
}

type PipelineDiagnostic = {
  id: string
  message: string
  nodeId?: string
  severity: DiagnosticSeverity
}

type PipelineNodeViewModel = {
  id: string
  description: string
  factoryName: string
  instanceName?: string
  kind: PipelineNodeKind
  position: {
    x: number
    y: number
  }
  properties: PipelineProperty[]
  sourceSpan?: SourceSpan
  tags: string[]
  tone: PipelineNodeTone
  warnings: string[]
}

type PipelineEdgeViewModel = {
  id: string
  label?: string
  sourceNodeId: string
  targetNodeId: string
}

type PipelineGraphViewModel = {
  edges: PipelineEdgeViewModel[]
  nodes: PipelineNodeViewModel[]
}

type PipelineParserStatus = 'empty' | 'placeholder' | 'parsed'

type PipelineDocumentViewModel = {
  id: string
  diagnostics: PipelineDiagnostic[]
  graph: PipelineGraphViewModel
  normalizedTextPreview: string
  parserStatus: PipelineParserStatus
  sourceKind: PipelineSourceKind
  sourceLabel: string
  subtitle: string
  title: string
}

export type {
  PipelineDiagnostic,
  PipelineDocumentViewModel,
  PipelineEdgeViewModel,
  PipelineGraphViewModel,
  PipelineNodeKind,
  PipelineNodeTone,
  PipelineNodeViewModel,
  PipelineProperty,
  PipelineSourceKind,
  SourceSpan,
}
