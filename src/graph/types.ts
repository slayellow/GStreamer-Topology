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
  end?: number
  lineStart: number
  lineEnd: number
  start?: number
}

type PipelineProperty = {
  key: string
  value: string
}

type PipelinePortKind = 'src' | 'sink' | 'named' | 'request'

type PipelinePortViewModel = {
  id: string
  kind: PipelinePortKind
  name: string
  nodeId: string
}

type PipelineDiagnostic = {
  id: string
  message: string
  nodeId?: string
  severity: DiagnosticSeverity
  sourceSpan?: SourceSpan
}

type PipelineNodeViewModel = {
  id: string
  description: string
  factoryName: string
  instanceName?: string
  kind: PipelineNodeKind
  dimensions?: {
    width: number
    height: number
  }
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
  sourcePort?: PipelinePortViewModel
  sourceNodeId: string
  targetPort?: PipelinePortViewModel
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
  normalizedText: string
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
  PipelinePortKind,
  PipelinePortViewModel,
  PipelineProperty,
  PipelineSourceKind,
  SourceSpan,
}
