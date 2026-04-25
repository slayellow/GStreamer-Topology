import { useState } from 'react'
import { InspectorPanel } from '../inspector/InspectorPanel.tsx'
import { GraphCanvas } from '../../graph/GraphCanvas.tsx'
import { DiagnosticsPanel } from './DiagnosticsPanel.tsx'
import type {
  PipelineDocumentViewModel,
  PipelineNodeViewModel,
} from '../../graph/types.ts'

type WorkspaceShellProps = {
  document: PipelineDocumentViewModel
  onBackHome: () => void
}

function findNode(
  document: PipelineDocumentViewModel,
  nodeId: string | null,
): PipelineNodeViewModel | null {
  if (!nodeId) {
    return null
  }

  return document.graph.nodes.find((node) => node.id === nodeId) ?? null
}

function getParserStatusLabel(status: PipelineDocumentViewModel['parserStatus']) {
  switch (status) {
    case 'parsed':
      return '파싱 완료'
    case 'placeholder':
      return '그래프 없음'
    case 'empty':
      return '미파싱'
  }
}

function WorkspaceShell({
  document,
  onBackHome,
}: WorkspaceShellProps) {
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(
    document.graph.nodes[0]?.id ?? null,
  )
  const [isDiagnosticsOpen, setIsDiagnosticsOpen] = useState(
    document.diagnostics.some((diagnostic) => diagnostic.severity !== 'info'),
  )

  const selectedNode = findNode(document, selectedNodeId)

  return (
    <main className="app-shell">
      <section className="workspace-shell">
        <header className="workspace-topbar panel">
          <div className="workspace-topbar__meta">
            <button className="secondary-button" onClick={onBackHome} type="button">
              다른 파이프라인 열기
            </button>
            <div className="workspace-topbar__title">
              <div className="eyebrow">워크스페이스</div>
              <h1>{document.title}</h1>
              <p className="muted-copy">{document.subtitle}</p>
            </div>
          </div>

          <div className="workspace-topbar__actions">
            <span className="card-chip">{document.sourceLabel}</span>
            <span className="card-chip">
              노드 {document.graph.nodes.length}개 / 엣지 {document.graph.edges.length}개
            </span>
            <span className="card-chip">{getParserStatusLabel(document.parserStatus)}</span>
            <button
              className={
                isDiagnosticsOpen
                  ? 'secondary-button diagnostics-toggle is-active'
                  : 'secondary-button diagnostics-toggle'
              }
              onClick={() => setIsDiagnosticsOpen((current) => !current)}
              type="button"
            >
              진단 {document.diagnostics.length}개
            </button>
          </div>
        </header>

        <div className="workspace-grid">
          <section className="workspace-main">
            <GraphCanvas
              document={document}
              selectedNodeId={selectedNodeId}
              onSelectNode={setSelectedNodeId}
            />
            <DiagnosticsPanel
              diagnostics={document.diagnostics}
              isOpen={isDiagnosticsOpen}
              onToggle={() => setIsDiagnosticsOpen((current) => !current)}
            />
          </section>

          <InspectorPanel document={document} selectedNode={selectedNode} />
        </div>
      </section>
    </main>
  )
}

export { WorkspaceShell }
