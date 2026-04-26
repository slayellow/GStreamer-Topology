import { useEffect, useState } from 'react'
import {
  inspectLocalElement,
  inspectRemoteElement,
  isTauriRuntime,
  type ElementMetadataResponse,
  type RemoteTargetInput,
} from '../../app/backend.ts'
import type { GStreamerRuntimeStatus } from '../../app/status.ts'
import { ConnectionBadge } from '../../components/ConnectionBadge.tsx'
import { IconButton } from '../../components/IconButton.tsx'
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
  gstreamerStatus: GStreamerRuntimeStatus
  remoteTarget: RemoteTargetInput | null
  onBackHome: () => void
}

type WorkspacePanel = 'diagnostics' | 'inspector' | 'source'

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
  gstreamerStatus,
  remoteTarget,
  onBackHome,
}: WorkspaceShellProps) {
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(
    document.graph.nodes[0]?.id ?? null,
  )
  const [activePanel, setActivePanel] = useState<WorkspacePanel | null>(null)
  const [metadataByFactory, setMetadataByFactory] = useState<Record<string, MetadataEntry>>({})

  const selectedNode = findNode(document, selectedNodeId)
  const syntaxDiagnostics = document.diagnostics.filter(
    (diagnostic) => diagnostic.severity !== 'info',
  )
  const selectedMetadata = selectedNode
    ? metadataByFactory[selectedNode.factoryName]
    : undefined
  const metadataAuthority =
    remoteTarget ? '원격 GStreamer' : '로컬 GStreamer'
  const apiStatus = remoteTarget
    ? {
        ...gstreamerStatus.remote,
        host: undefined,
        message: gstreamerStatus.remote.version
          ? '원격 GStreamer API 연결됨'
          : gstreamerStatus.remote.message,
        port: undefined,
      }
    : gstreamerStatus.local

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
      remoteTarget
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
  }, [metadataByFactory, remoteTarget, selectedNode])

  function togglePanel(panel: WorkspacePanel) {
    setActivePanel((current) => (current === panel ? null : panel))
  }

  return (
    <main className="app-shell app-shell--workspace">
      <section className="workspace-shell">
        <header className="workspace-topbar panel">
          <div className="workspace-topbar__meta">
            <IconButton icon="arrowLeft" label="첫 화면으로 돌아가기" onClick={onBackHome} />
            <div className="workspace-topbar__title">
              <div className="eyebrow">워크스페이스</div>
              <h1>{document.title}</h1>
              <p className="muted-copy">{document.subtitle}</p>
            </div>
          </div>

          <div className="workspace-topbar__actions">
            <ConnectionBadge label="GStreamer API" status={apiStatus} />
            {remoteTarget ? (
              <ConnectionBadge label="Remote Server" status={gstreamerStatus.remote} />
            ) : null}
            <span className="card-chip">{document.sourceLabel}</span>
            <span className="card-chip">
              노드 {document.graph.nodes.length}개 / 엣지 {document.graph.edges.length}개
            </span>
            <span className="card-chip">{getParserStatusLabel(document.parserStatus)}</span>
            <span className="card-chip">{metadataAuthority}</span>
            <div className="workspace-topbar__panel-actions" aria-label="보조 패널">
              <IconButton
                active={activePanel === 'inspector'}
                badge={selectedNode ? '•' : undefined}
                icon="panelRight"
                label="인스펙터 열기"
                onClick={() => togglePanel('inspector')}
              />
              <IconButton
                active={activePanel === 'source'}
                icon="fileText"
                label="Pipeline 원문 열기"
                onClick={() => togglePanel('source')}
              />
              <IconButton
                active={activePanel === 'diagnostics'}
                badge={document.diagnostics.length || undefined}
                icon="diagnostics"
                label="파서 진단 열기"
                onClick={() => togglePanel('diagnostics')}
              />
            </div>
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

        {gstreamerStatus.local.state === 'failed' && !remoteTarget ? (
          <aside className="workspace-alert severity-info">
            <strong>로컬 GStreamer 정보 없음</strong>
            <span>
              {gstreamerStatus.local.message ?? '이 장비에서 `gst-inspect-1.0`을 찾지 못했습니다.'}
              {' '}
              토폴로지는 텍스트 파서
              기준으로 계속 표시하고, Element 내부 정보는 설치 후 확인할 수 있습니다.
            </span>
          </aside>
        ) : null}

        <div className={activePanel ? 'workspace-grid workspace-grid--drawer-open' : 'workspace-grid'}>
          <section className="workspace-main">
            <GraphCanvas
              document={document}
              selectedNodeId={selectedNodeId}
              onSelectNode={setSelectedNodeId}
            />
          </section>

          {activePanel ? (
            <aside className="workspace-drawer">
              <div className="workspace-drawer__header">
                <span className="card-chip muted-chip">
                  {activePanel === 'inspector'
                    ? '인스펙터'
                    : activePanel === 'source'
                      ? 'Pipeline 원문'
                      : '파서 진단'}
                </span>
                <IconButton icon="close" label="패널 닫기" onClick={() => setActivePanel(null)} />
              </div>

              {activePanel === 'inspector' ? (
                <InspectorPanel
                  document={document}
                  metadata={selectedMetadata}
                  selectedNode={selectedNode}
                />
              ) : null}

              {activePanel === 'source' ? (
                <SourceTextPanel
                  document={document}
                  isOpen
                  selectedNode={selectedNode}
                  onToggle={() => setActivePanel(null)}
                />
              ) : null}

              {activePanel === 'diagnostics' ? (
                <DiagnosticsPanel
                  diagnostics={document.diagnostics}
                  isOpen
                  onToggle={() => setActivePanel(null)}
                />
              ) : null}
            </aside>
          ) : null}
        </div>
      </section>
    </main>
  )
}

export { WorkspaceShell }
