import { useLayoutEffect, useRef } from 'react'
import type {
  PipelineDiagnostic,
  PipelineDocumentViewModel,
  PipelineNodeViewModel,
  SourceSpan,
} from '../../graph/types.ts'

type SourceTextPanelProps = {
  document: PipelineDocumentViewModel
  isOpen: boolean
  selectedNode: PipelineNodeViewModel | null
  selectionRevision: number
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

function SourceTextPanel({
  document,
  isOpen,
  selectedNode,
  selectionRevision,
  onToggle,
}: SourceTextPanelProps) {
  const highlightRef = useRef<HTMLElement | null>(null)
  const codeScrollRef = useRef<HTMLPreElement | null>(null)
  const text = document.normalizedText
  const selectedSpan = validSpan(text, selectedNode?.sourceSpan)
  const diagnostics = syntaxDiagnostics(document.diagnostics)
  const showHighlightFallback = Boolean(selectedNode && !selectedSpan)

  const highlightedText = selectedSpan
    ? {
        before: text.slice(0, selectedSpan.start),
        selected: selectedSpan.selected,
        after: text.slice(selectedSpan.end),
      }
    : { before: text, selected: '', after: '' }

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
          {selectedNode?.sourceSpan ? (
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
              <span>{highlightedText.before}</span>
              {highlightedText.selected ? (
                <mark ref={highlightRef}>{highlightedText.selected}</mark>
              ) : null}
              <span>{highlightedText.after}</span>
            </code>
          </pre>
        </div>
      ) : null}
    </section>
  )
}

export { SourceTextPanel }
