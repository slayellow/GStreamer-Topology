import { useEffect, useState } from 'react'
import {
  inspectLocalElement,
  inspectRemoteElement,
  isTauriRuntime,
  probeLocalGStreamer,
  type ElementMetadataResponse,
  type GStreamerProbeResponse,
  type RemoteTargetInput,
} from '../../app/backend.ts'
import { InspectorPanel } from '../inspector/InspectorPanel.tsx'
import { GraphCanvas } from '../../graph/GraphCanvas.tsx'
import { DiagnosticsPanel } from './DiagnosticsPanel.tsx'
import { SourceTextPanel } from './SourceTextPanel.tsx'
import type {
  PipelineDocumentViewModel,
  PipelineNodeViewModel,
} from '../../graph/types.ts'

type WorkspaceShellProps = {
  document: PipelineDocumentViewModel
  remoteTarget: RemoteTargetInput | null
  onBackHome: () => void
}

type MetadataEntry = {
  data?: ElementMetadataResponse
  message?: string
  status: 'loading' | 'ready' | 'unavailable'
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
  remoteTarget,
  onBackHome,
}: WorkspaceShellProps) {
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(
    document.graph.nodes[0]?.id ?? null,
  )
  const [isDiagnosticsOpen, setIsDiagnosticsOpen] = useState(
    document.diagnostics.some((diagnostic) => diagnostic.severity !== 'info'),
  )
  const [isSourceOpen, setIsSourceOpen] = useState(false)
  const [localProbe, setLocalProbe] = useState<GStreamerProbeResponse | null>(null)
  const [metadataByFactory, setMetadataByFactory] = useState<Record<string, MetadataEntry>>({})

  const selectedNode = findNode(document, selectedNodeId)
  const syntaxDiagnostics = document.diagnostics.filter(
    (diagnostic) => diagnostic.severity !== 'info',
  )
  const selectedMetadata = selectedNode
    ? metadataByFactory[selectedNode.factoryName]
    : undefined
  const metadataAuthority =
    document.sourceKind === 'remote_file' ? '원격 GStreamer' : '로컬 GStreamer'

  useEffect(() => {
    if (!isTauriRuntime() || document.sourceKind === 'remote_file') {
      return
    }

    let isCancelled = false
    probeLocalGStreamer()
      .then((response) => {
        if (!isCancelled) {
          setLocalProbe(response)
        }
      })
      .catch(() => {
        if (!isCancelled) {
          setLocalProbe({
            available: false,
            authority: 'local',
            diagnostic: '로컬 GStreamer 확인에 실패했습니다.',
            version_output: null,
          })
        }
      })

    return () => {
      isCancelled = true
    }
  }, [document.id, document.sourceKind])

  useEffect(() => {
    if (!selectedNode || !isTauriRuntime()) {
      return
    }

    const key = selectedNode.factoryName
    if (metadataByFactory[key]) {
      return
    }

    let isCancelled = false
    const loader =
      document.sourceKind === 'remote_file' && remoteTarget
        ? inspectRemoteElement(remoteTarget, selectedNode.factoryName)
        : inspectLocalElement(selectedNode.factoryName)

    loader
      .then((metadata) => {
        if (isCancelled) {
          return
        }

        setMetadataByFactory((current) => ({
          ...current,
          [key]: {
            data: metadata,
            message: metadata.diagnostic ?? undefined,
            status: metadata.available ? 'ready' : 'unavailable',
          },
        }))
      })
      .catch((error) => {
        if (isCancelled) {
          return
        }

        console.error(error)
        setMetadataByFactory((current) => ({
          ...current,
          [key]: {
            message: 'Element 메타데이터를 가져오지 못했습니다.',
            status: 'unavailable',
          },
        }))
      })

    return () => {
      isCancelled = true
    }
  }, [document.sourceKind, metadataByFactory, remoteTarget, selectedNode])

  return (
    <main className="app-shell app-shell--workspace">
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
            <span className="card-chip">{metadataAuthority}</span>
            <button
              className={
                isSourceOpen
                  ? 'secondary-button diagnostics-toggle is-active'
                  : 'secondary-button diagnostics-toggle'
              }
              onClick={() => setIsSourceOpen((current) => !current)}
              type="button"
            >
              Pipeline 원문
            </button>
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

        {syntaxDiagnostics.length ? (
          <aside className="workspace-alert severity-warning">
            <strong>파이프라인 구문 확인 필요</strong>
            <span>
              토폴로지는 생성됐지만 파서 진단 {syntaxDiagnostics.length}건이 있습니다.
              Element 연결의 의미 검증은 이후 단계에서 보강하고, 현재는 원문과 진단을
              함께 확인해 주세요.
            </span>
          </aside>
        ) : null}

        {localProbe && !localProbe.available ? (
          <aside className="workspace-alert severity-info">
            <strong>로컬 GStreamer 정보 없음</strong>
            <span>
              이 장비에서 `gst-inspect-1.0`을 찾지 못했습니다. 토폴로지는 텍스트 파서
              기준으로 계속 표시하고, Element 내부 정보는 설치 후 확인할 수 있습니다.
            </span>
          </aside>
        ) : null}

        <div className="workspace-grid">
          <section className="workspace-main">
            <GraphCanvas
              document={document}
              selectedNodeId={selectedNodeId}
              onSelectNode={setSelectedNodeId}
            />
            <SourceTextPanel
              document={document}
              isOpen={isSourceOpen}
              selectedNode={selectedNode}
              onToggle={() => setIsSourceOpen((current) => !current)}
            />
            <DiagnosticsPanel
              diagnostics={document.diagnostics}
              isOpen={isDiagnosticsOpen}
              onToggle={() => setIsDiagnosticsOpen((current) => !current)}
            />
          </section>

          <InspectorPanel
            document={document}
            metadata={selectedMetadata}
            selectedNode={selectedNode}
          />
        </div>
      </section>
    </main>
  )
}

export { WorkspaceShell }
