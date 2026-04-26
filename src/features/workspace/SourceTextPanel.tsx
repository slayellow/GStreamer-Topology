import { useEffect, useRef } from 'react'
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
  onToggle: () => void
}

const encoder = new TextEncoder()

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

function clampSpan(text: string, span?: SourceSpan) {
  if (!span || typeof span.start !== 'number' || typeof span.end !== 'number') {
    return null
  }

  const start = byteOffsetToStringIndex(text, Math.max(0, span.start))
  const end = byteOffsetToStringIndex(text, Math.max(span.start, span.end))

  return {
    start: Math.min(start, text.length),
    end: Math.min(Math.max(end, start), text.length),
  }
}

function syntaxDiagnostics(diagnostics: PipelineDiagnostic[]) {
  return diagnostics.filter((diagnostic) => diagnostic.severity !== 'info')
}

function SourceTextPanel({
  document,
  isOpen,
  selectedNode,
  onToggle,
}: SourceTextPanelProps) {
  const highlightRef = useRef<HTMLElement | null>(null)
  const text = document.normalizedText
  const selectedSpan = clampSpan(text, selectedNode?.sourceSpan)
  const diagnostics = syntaxDiagnostics(document.diagnostics)

  const highlightedText = selectedSpan
    ? {
        before: text.slice(0, selectedSpan.start),
        selected: text.slice(selectedSpan.start, selectedSpan.end),
        after: text.slice(selectedSpan.end),
      }
    : { before: text, selected: '', after: '' }

  useEffect(() => {
    if (!isOpen || !highlightRef.current) {
      return
    }

    highlightRef.current.scrollIntoView({
      block: 'center',
      behavior: 'smooth',
    })
  }, [isOpen, selectedNode?.id])

  return (
    <section className={isOpen ? 'source-panel is-open' : 'source-panel'}>
      <button className="source-panel__toggle" onClick={onToggle} type="button">
        <span>
          <strong>Pipeline 원문</strong>
          {selectedNode?.sourceSpan ? (
            <em>
              선택 위치: {selectedNode.sourceSpan.lineStart}-{selectedNode.sourceSpan.lineEnd}행
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

          <pre className="source-panel__code">
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
