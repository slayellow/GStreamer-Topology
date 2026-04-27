import { invoke } from '@tauri-apps/api/core'

type TauriWindow = Window & {
  __TAURI_INTERNALS__?: unknown
}

type BackendSourceKind = 'local_file' | 'remote_file' | 'pasted_text'
type BackendDiagnosticSeverity = 'info' | 'warning' | 'error'
type BackendPipelineNodeKind = 'element' | 'unknown'
type BackendPipelinePortKind = 'src' | 'sink' | 'named' | 'request'

export type BackendSourceSpan = {
  start: number
  end: number
}

export type BackendPipelineProperty = {
  key: string
  value: string
}

export type BackendParseDiagnostic = {
  severity: BackendDiagnosticSeverity
  code: string
  message: string
  span?: BackendSourceSpan | null
}

export type BackendPipelineNode = {
  id: string
  factory_name: string
  instance_name?: string | null
  kind: BackendPipelineNodeKind
  properties: BackendPipelineProperty[]
  source_span: BackendSourceSpan
}

export type BackendPipelinePort = {
  id: string
  node_id: string
  port_kind: BackendPipelinePortKind
  port_name: string
}

export type BackendPipelineEdge = {
  id: string
  source_node_id: string
  source_port?: BackendPipelinePort | null
  target_node_id: string
  target_port?: BackendPipelinePort | null
  caps_label?: string | null
  source_span: BackendSourceSpan
}

export type BackendPipelineDocument = {
  source_kind: BackendSourceKind
  path?: string | null
  source_name?: string | null
  raw_text: string
  normalized_text: string
  diagnostics: BackendParseDiagnostic[]
  graph: {
    nodes: BackendPipelineNode[]
    edges: BackendPipelineEdge[]
  }
}

export type RemoteTargetInput = {
  host: string
  port: number
  username: string
  password: string
}

export type RemoteProbeResponse = {
  host: string
  port: number
  username: string
  version_output: string
  sample_element_output?: string | null
}

type MetadataAuthority = 'local' | 'remote'

export type GStreamerProbeResponse = {
  available: boolean
  authority: MetadataAuthority
  version_output?: string | null
  diagnostic?: string | null
}

export type ElementPropertyMetadata = {
  current_value?: string | null
  default_value?: string | null
  description?: string | null
  name: string
  value_type?: string | null
}

export type ElementPadTemplateMetadata = {
  name: string
  direction: string
  presence?: string | null
}

export type ElementMetadataResponse = {
  available: boolean
  authority: MetadataAuthority
  factory_name: string
  long_name?: string | null
  klass?: string | null
  description?: string | null
  plugin_name?: string | null
  properties: ElementPropertyMetadata[]
  pad_templates: ElementPadTemplateMetadata[]
  raw_output?: string | null
  diagnostic?: string | null
}

export function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in (window as TauriWindow)
}

export async function parsePipelineText(rawText: string, sourceName?: string) {
  return invoke<BackendPipelineDocument>('parse_pipeline_text', { rawText, sourceName })
}

export async function loadLocalPipelineFile(path: string) {
  return invoke<BackendPipelineDocument>('load_local_pipeline_file', { path })
}

export async function saveExportFile(
  path: string,
  contents: string,
) {
  return invoke<string | null>('save_export_file', { path, contents })
}

export async function saveExportFileToDownloads(
  fileName: string,
  contents: string,
) {
  return invoke<string>('save_export_file_to_downloads', { fileName, contents })
}

export async function suggestExportFilePath(fileName: string) {
  return invoke<string>('suggest_export_file_path', { fileName })
}

export async function probeLocalGStreamer() {
  return invoke<GStreamerProbeResponse>('probe_local_gstreamer')
}

export async function inspectLocalElement(factoryName: string) {
  return invoke<ElementMetadataResponse>('inspect_local_element', {
    factoryName,
  })
}

export async function probeRemoteTarget(
  request: RemoteTargetInput,
  sampleElement?: string,
) {
  return invoke<RemoteProbeResponse>('probe_remote_target', {
    request: {
      ...request,
      port: Number(request.port || 22),
    },
    sampleElement,
  })
}

export async function loadRemotePipeline(
  request: RemoteTargetInput,
  path: string,
) {
  return invoke<BackendPipelineDocument>('load_remote_pipeline', {
    request: {
      ...request,
      port: Number(request.port || 22),
    },
    path,
  })
}

export async function inspectRemoteElement(
  request: RemoteTargetInput,
  factoryName: string,
) {
  return invoke<ElementMetadataResponse>('inspect_remote_element', {
    request: {
      ...request,
      port: Number(request.port || 22),
    },
    factoryName,
  })
}
