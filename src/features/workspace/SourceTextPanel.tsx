import { useLayoutEffect, useMemo, useRef } from 'react'
import type {
  PipelineDiagnostic,
  PipelineDocumentViewModel,
  PipelineNodeViewModel,
  SourceSpan,
} from '../../graph/types.ts'

type SourceTextPanelProps = {
  document: PipelineDocumentViewModel
  focusedSource?: {
    id: string
    label: string
    span: SourceSpan
  } | null
  isOpen: boolean
  selectedNode: PipelineNodeViewModel | null
  selectionRevision: number
  onSelectNodeSource: (nodeId: string) => void
  onToggle: () => void
}

const encoder = new TextEncoder()

function selectedNodeLabel(node: PipelineNodeViewModel) {
  return node.instanceName
    ? `${node.instanceName} (${node.factoryName})`
    : node.factoryName
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

function isFiniteNumber(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value)
}

function validSpan(text: string, span?: SourceSpan) {
  if (!span || !isFiniteNumber(span.start) || !isFiniteNumber(span.end)) {
    return null
  }

  const textByteLength = encoder.encode(text).length
  if (span.start < 0 || span.start >= span.end || span.end > textByteLength) {
    return null
  }

  const start = byteOffsetToStringIndex(text, span.start)
  const end = byteOffsetToStringIndex(text, span.end)
  const selected = text.slice(start, end)
  if (!selected.trim()) {
    return null
  }

  return {
    end,
    selected,
    start,
  }
}

function syntaxDiagnostics(diagnostics: PipelineDiagnostic[]) {
  return diagnostics.filter((diagnostic) => diagnostic.severity !== 'info')
}

type SourceNodeSpan = {
  end: number
  nodeId: string
  selected: string
  start: number
}

type SourceTextSegment = {
  end: number
  isHighlighted: boolean
  nodeId?: string
  start: number
  text: string
}

function nodeSourceSpans(text: string, nodes: PipelineNodeViewModel[]) {
  return nodes
    .map((node) => {
      const span = validSpan(text, node.sourceSpan)
      return span
        ? {
            ...span,
            nodeId: node.id,
          }
        : null
    })
    .filter((span): span is SourceNodeSpan => Boolean(span))
    .sort((first, second) => first.start - second.start || first.end - second.end)
}

function sourceTextSegments(
  text: string,
  nodes: SourceNodeSpan[],
  highlightedSpan: ReturnType<typeof validSpan>,
) {
  const boundaries = new Set([0, text.length])

  for (const node of nodes) {
    boundaries.add(node.start)
    boundaries.add(node.end)
  }

  if (highlightedSpan) {
    boundaries.add(highlightedSpan.start)
    boundaries.add(highlightedSpan.end)
  }

  const sortedBoundaries = [...boundaries].sort((first, second) => first - second)
  const segments: SourceTextSegment[] = []

  for (let index = 0; index < sortedBoundaries.length - 1; index += 1) {
    const start = sortedBoundaries[index]
    const end = sortedBoundaries[index + 1]
    if (start >= end) {
      continue
    }

    const node = nodes.find((candidate) => start >= candidate.start && end <= candidate.end)
    segments.push({
      end,
      isHighlighted: Boolean(
        highlightedSpan
          && start >= highlightedSpan.start
          && end <= highlightedSpan.end,
      ),
      nodeId: node?.nodeId,
      start,
      text: text.slice(start, end),
    })
  }

  return segments
}

function SourceTextPanel({
  document,
  focusedSource,
  isOpen,
  selectedNode,
  selectionRevision,
  onSelectNodeSource,
  onToggle,
}: SourceTextPanelProps) {
  const highlightRef = useRef<HTMLElement | null>(null)
  const codeScrollRef = useRef<HTMLPreElement | null>(null)
  const text = document.normalizedText
  const activeSpan = focusedSource?.span ?? selectedNode?.sourceSpan
  const selectedSpan = useMemo(() => validSpan(text, activeSpan), [activeSpan, text])
  const clickableSpans = useMemo(
    () => nodeSourceSpans(text, document.graph.nodes),
    [document.graph.nodes, text],
  )
  const textSegments = useMemo(
    () => sourceTextSegments(text, clickableSpans, selectedSpan),
    [clickableSpans, selectedSpan, text],
  )
  const highlightSegmentKey = textSegments.find((segment) => segment.isHighlighted)
  const firstHighlightKey = highlightSegmentKey
    ? `${highlightSegmentKey.start}-${highlightSegmentKey.end}`
    : null
  const diagnostics = syntaxDiagnostics(document.diagnostics)
  const showHighlightFallback = Boolean((selectedNode || focusedSource) && !selectedSpan)

  useLayoutEffect(() => {
    if (!isOpen) {
      return
    }

    const frameId = window.requestAnimationFrame(() => {
      const mark = highlightRef.current
      const scroller = codeScrollRef.current
      if (!mark || !scroller) {
        return
      }

      const markRect = mark.getBoundingClientRect()
      const scrollerRect = scroller.getBoundingClientRect()
      const targetTop = scroller.scrollTop
        + markRect.top
        - scrollerRect.top
        - (scroller.clientHeight / 2)
        + (markRect.height / 2)
      const targetLeft = scroller.scrollLeft
        + markRect.left
        - scrollerRect.left
        - (scroller.clientWidth / 2)
        + (markRect.width / 2)

      scroller.scrollTo({
        left: Math.max(0, targetLeft),
        top: Math.max(0, targetTop),
        behavior: 'auto',
      })
    })

    return () => window.cancelAnimationFrame(frameId)
  }, [
    document.id,
    focusedSource?.id,
    isOpen,
    selectedNode?.id,
    selectedSpan?.end,
    selectedSpan?.start,
    selectionRevision,
  ])

  return (
    <section className={isOpen ? 'source-panel is-open' : 'source-panel'}>
      <button className="source-panel__toggle" onClick={onToggle} type="button">
        <span>
          <strong>Pipeline 원문</strong>
          {focusedSource ? (
            <em>
              진단 위치: {focusedSource.span.lineStart}-{focusedSource.span.lineEnd}행 ·
              {' '}
              {focusedSource.label}
            </em>
          ) : selectedNode?.sourceSpan ? (
            <em>
              선택: {selectedNodeLabel(selectedNode)} · {selectedNode.sourceSpan.lineStart}-
              {selectedNode.sourceSpan.lineEnd}행
            </em>
          ) : (
            <em>선택한 Element의 원문 위치를 함께 확인합니다.</em>
          )}
        </span>
        <span className="card-chip">{isOpen ? '접기' : '보기'}</span>
      </button>

      {isOpen ? (
        <div className="source-panel__body">
          {diagnostics.length ? (
            <div className="source-panel__alert">
              파서 진단 {diagnostics.length}건이 있습니다. 원문과 캔버스를 함께 보며
              연결 상태를 확인하세요.
            </div>
          ) : null}

          {showHighlightFallback ? (
            <div className="source-panel__alert source-panel__alert--highlight">
              원문 위치를 찾지 못했습니다. 그래프는 유지되지만 이 Element의 source
              span을 확인해야 합니다.
            </div>
          ) : null}

          <pre className="source-panel__code" ref={codeScrollRef}>
            <code>
              {textSegments.map((segment) => {
                const segmentKey = `${segment.start}-${segment.end}`
                const content = segment.isHighlighted ? (
                  <mark ref={segmentKey === firstHighlightKey ? highlightRef : undefined}>
                    {segment.text}
                  </mark>
                ) : (
                  segment.text
                )

                if (segment.nodeId) {
                  const nodeId = segment.nodeId
                  return (
                    <span
                      className={[
                        'source-panel__node-token',
                        segment.isHighlighted ? 'is-highlighted' : '',
                      ].filter(Boolean).join(' ')}
                      key={segmentKey}
                      onClick={() => onSelectNodeSource(nodeId)}
                      onKeyDown={(event) => {
                        if (event.key !== 'Enter' && event.key !== ' ') {
                          return
                        }

                        event.preventDefault()
                        onSelectNodeSource(nodeId)
                      }}
                      role="button"
                      tabIndex={0}
                      title="캔버스에서 이 Element 선택"
                    >
                      {content}
                    </span>
                  )
                }

                return <span key={segmentKey}>{content}</span>
              })}
            </code>
          </pre>
        </div>
      ) : null}
    </section>
  )
}

export { SourceTextPanel }
