import ELK from 'elkjs/lib/elk.bundled.js'
import type {
  BackendParseDiagnostic,
  BackendPipelineDocument,
  BackendPipelineNode,
  BackendPipelinePort,
  BackendSourceSpan,
} from '../app/backend.ts'
import type {
  PipelineDiagnostic,
  PipelineDocumentViewModel,
  PipelineGraphViewModel,
  PipelinePortViewModel,
  PipelineNodeTone,
  PipelineNodeViewModel,
} from './types.ts'

const elk = new ELK()
const DEFAULT_NODE_HEIGHT = 118
const DEFAULT_NODE_WIDTH = 220
const encoder = new TextEncoder()

function sourceKindLabel(sourceKind: BackendPipelineDocument['source_kind']) {
  switch (sourceKind) {
    case 'local_file':
      return '로컬 파일'
    case 'remote_file':
      return '원격 파일'
    case 'pasted_text':
      return '붙여넣은 텍스트'
  }
}

function severityLabel(severity: BackendParseDiagnostic['severity']) {
  return severity
}

function localizeDiagnosticMessage(diagnostic: BackendParseDiagnostic) {
  switch (diagnostic.code) {
    case 'duplicate-instance-name': {
      const match = diagnostic.message.match(/`([^`]+)`/)
      return match
        ? `이름이 \`${match[1]}\` 인 요소가 두 번 선언되었습니다.`
        : '같은 이름의 요소가 두 번 선언되었습니다.'
    }
    case 'unparsed-element-token': {
      const match = diagnostic.message.match(/Ignored tokens on element `([^`]+)`: (.+)$/)
      return match
        ? `요소 \`${match[1]}\` 에서 해석하지 못한 토큰: ${match[2]}`
        : '요소 구문의 일부 토큰을 해석하지 못했습니다.'
    }
    case 'empty-component':
      return '비어 있는 파이프라인 구성 요소를 발견했습니다.'
    case 'dangling-caps':
      return '어느 파이프라인 세그먼트에도 연결되지 않은 caps 구문을 발견했습니다.'
    case 'missing-link-operator': {
      const match = diagnostic.message.match(/`([^`]+)`/)
      return match
        ? `토큰 \`${match[1]}\` 앞에 파이프라인 연결 연산자 \`!\` 가 없는 것으로 보입니다.`
        : '요소 사이에 파이프라인 연결 연산자 `!` 가 없는 것으로 보입니다.'
    }
    case 'unresolved-reference': {
      const match = diagnostic.message.match(/`([^`]+)`/)
      return match
        ? `이름이 지정된 참조 \`${match[1]}\` 를 선언된 요소와 연결하지 못했습니다.`
        : '이름이 지정된 참조를 선언된 요소와 연결하지 못했습니다.'
    }
    default:
      return '파이프라인을 해석하는 중 추가 진단이 보고되었습니다.'
  }
}

function defaultTitle(document: BackendPipelineDocument) {
  if (document.source_kind === 'pasted_text') {
    return '붙여넣은 파이프라인'
  }

  if (document.source_kind === 'remote_file') {
    return '원격 파이프라인'
  }

  return '이름 없는 파이프라인'
}

function defaultSourceLabel(sourceKind: BackendPipelineDocument['source_kind']) {
  switch (sourceKind) {
    case 'local_file':
      return '로컬 파이프라인'
    case 'remote_file':
      return '원격 파이프라인'
    case 'pasted_text':
      return '파이프라인 텍스트'
  }
}

function countLineBreaks(text: string, index: number) {
  let line = 1
  for (let i = 0; i < index && i < text.length; i += 1) {
    if (text[i] === '\n') {
      line += 1
    }
  }
  return line
}

function byteOffsetToStringIndex(text: string, byteOffset: number) {
  let bytes = 0
  let index = 0

  for (const char of text) {
    const nextBytes = bytes + encoder.encode(char).length
    if (nextBytes > byteOffset) {
      return index
    }

    bytes = nextBytes
    index += char.length
  }

  return text.length
}

function toLineSpan(text: string, span: BackendSourceSpan | undefined | null) {
  if (!span) {
    return undefined
  }

  const startIndex = byteOffsetToStringIndex(text, Math.max(0, span.start))
  const endIndex = byteOffsetToStringIndex(text, Math.max(span.start, span.end))

  return {
    start: span.start,
    end: span.end,
    lineStart: countLineBreaks(text, startIndex),
    lineEnd: countLineBreaks(text, endIndex),
  }
}

function summarizeNode(node: BackendPipelineNode) {
  const name = node.instance_name || node.factory_name
  const propertySummary = node.properties
    .slice(0, 2)
    .map((property) => `${property.key}=${property.value}`)
    .join(', ')

  return propertySummary
    ? `${name} 요소 · ${propertySummary}`
    : `${name} 요소가 파이프라인 텍스트에서 파싱되었습니다.`
}

function classifyTone(node: BackendPipelineNode): PipelineNodeTone {
  if (node.kind === 'unknown') {
    return 'unknown'
  }

  const factory = node.factory_name.toLowerCase()

  if (
    factory.endsWith('src') ||
    factory.includes('camera') ||
    factory === 'appsrc'
  ) {
    return 'source'
  }

  if (
    factory === 'tee' ||
    factory.includes('selector')
  ) {
    return 'branch'
  }

  if (
    factory === 'funnel' ||
    factory.includes('mux') ||
    factory.includes('composer') ||
    factory.includes('compositor')
  ) {
    return 'merge'
  }

  if (
    factory.includes('ml') ||
    factory.includes('snpe') ||
    factory.includes('detect') ||
    factory.includes('tracker')
  ) {
    return 'ai'
  }

  if (
    factory.endsWith('sink') ||
    factory.includes('enc') ||
    factory.includes('pay') ||
    factory.includes('parse')
  ) {
    return 'output'
  }

  return 'utility'
}

function toneTagLabel(tone: PipelineNodeTone) {
  switch (tone) {
    case 'source':
      return '소스'
    case 'branch':
      return '분기'
    case 'merge':
      return '병합'
    case 'ai':
      return 'AI'
    case 'output':
      return '출력'
    case 'utility':
      return '유틸리티'
    case 'unknown':
      return '알 수 없음'
  }
}

function nodeTags(node: BackendPipelineNode) {
  const tags = new Set<string>([
    toneTagLabel(classifyTone(node)),
    node.factory_name,
  ])

  if (node.instance_name) {
    tags.add(node.instance_name)
  }

  return Array.from(tags).slice(0, 4)
}

function nodeDimensions(node: BackendPipelineNode) {
  return {
    width: Math.max(DEFAULT_NODE_WIDTH, (node.instance_name || node.factory_name).length * 8 + 120),
    height: DEFAULT_NODE_HEIGHT,
  }
}

function portViewModel(
  port: BackendPipelinePort | null | undefined,
): PipelinePortViewModel | undefined {
  if (!port) {
    return undefined
  }

  return {
    id: port.id,
    kind: port.port_kind,
    name: port.port_name,
    nodeId: port.node_id,
  }
}

function meaningfulPortLabel(port: PipelinePortViewModel | undefined) {
  if (!port) {
    return undefined
  }

  const shouldShowName =
    port.kind === 'request' ||
    port.kind === 'named' ||
    (port.kind === 'sink' && port.name !== 'sink') ||
    (port.kind === 'src' && port.name !== 'src')

  return shouldShowName ? port.name : undefined
}

function edgeLabel(edge: BackendPipelineDocument['graph']['edges'][number]) {
  const sourcePort = portViewModel(edge.source_port)
  const targetPort = portViewModel(edge.target_port)
  const portLabel = [
    meaningfulPortLabel(sourcePort),
    meaningfulPortLabel(targetPort),
  ].filter(Boolean).join(' -> ')
  const details = [edge.caps_label, portLabel].filter(Boolean)

  return details.length ? details.join(' · ') : undefined
}

function buildWarnings(
  node: BackendPipelineNode,
  diagnostics: BackendParseDiagnostic[],
) {
  return diagnostics
    .filter((diagnostic) => {
      const span = diagnostic.span
      if (!span) {
        return false
      }

      return span.start <= node.source_span.end && span.end >= node.source_span.start
    })
    .map(localizeDiagnosticMessage)
}

async function layoutNodes(
  normalizedText: string,
  nodes: BackendPipelineNode[],
  edges: BackendPipelineDocument['graph']['edges'],
) {
  const maxEdgeLabelLength = Math.max(
    0,
    ...edges.map((edge) => edgeLabel(edge)?.length ?? 0),
  )
  const layerSpacing = Math.min(
    560,
    Math.max(104, 84 + maxEdgeLabelLength * 4),
  )

  const layout = await elk.layout({
    id: 'root',
    layoutOptions: {
      'elk.algorithm': 'layered',
      'elk.direction': 'RIGHT',
      'elk.edgeRouting': 'ORTHOGONAL',
      'elk.spacing.nodeNode': '36',
      'elk.layered.spacing.nodeNodeBetweenLayers': String(layerSpacing),
    },
    children: nodes.map((node) => ({
      id: node.id,
      ...nodeDimensions(node),
    })),
    edges: edges.map((edge) => ({
      id: edge.id,
      sources: [edge.source_node_id],
      targets: [edge.target_node_id],
    })),
  })

  const positions = new Map<string, { x: number; y: number }>()
  for (const child of layout.children ?? []) {
    positions.set(child.id, {
      x: child.x ?? 0,
      y: child.y ?? 0,
    })
  }

  const viewNodes: PipelineNodeViewModel[] = nodes.map((node) => ({
    id: node.id,
    description: summarizeNode(node),
    dimensions: nodeDimensions(node),
    factoryName: node.factory_name,
    instanceName: node.instance_name ?? undefined,
    kind: node.kind,
    position: positions.get(node.id) ?? { x: 0, y: 0 },
    properties: node.properties,
    sourceSpan: toLineSpan(normalizedText, node.source_span),
    tags: nodeTags(node),
    tone: classifyTone(node),
    warnings: buildWarnings(node, []),
  }))

  return viewNodes
}

export async function toViewModel(
  document: BackendPipelineDocument,
): Promise<PipelineDocumentViewModel> {
  const diagnostics: PipelineDiagnostic[] = document.diagnostics.map(
    (diagnostic, index) => ({
      id: `${diagnostic.code}-${index}`,
      severity: severityLabel(diagnostic.severity),
      message: localizeDiagnosticMessage(diagnostic),
      nodeId: undefined,
      sourceSpan: toLineSpan(document.normalized_text, diagnostic.span),
    }),
  )

  const graphNodes = await layoutNodes(
    document.normalized_text,
    document.graph.nodes,
    document.graph.edges,
  )

  const graph: PipelineGraphViewModel = {
    nodes: graphNodes.map((node) => ({
      ...node,
      warnings: buildWarnings(
        document.graph.nodes.find((candidate) => candidate.id === node.id)!,
        document.diagnostics,
      ),
    })),
    edges: document.graph.edges.map((edge) => ({
      id: edge.id,
      label: edgeLabel(edge),
      sourcePort: portViewModel(edge.source_port),
      sourceNodeId: edge.source_node_id,
      targetPort: portViewModel(edge.target_port),
      targetNodeId: edge.target_node_id,
    })),
  }

  return {
    id: `${document.source_kind}:${document.path ?? document.source_name ?? 'untitled'}`,
    title: document.source_name ?? document.path ?? defaultTitle(document),
    subtitle: `요소 ${document.graph.nodes.length}개를 ${sourceKindLabel(document.source_kind)}에서 렌더링했습니다.`,
    sourceKind: document.source_kind,
    sourceLabel: document.path ?? document.source_name ?? defaultSourceLabel(document.source_kind),
    parserStatus: graph.nodes.length ? 'parsed' : 'placeholder',
    normalizedText: document.normalized_text,
    normalizedTextPreview: document.normalized_text.slice(0, 340),
    diagnostics,
    graph,
  }
}
