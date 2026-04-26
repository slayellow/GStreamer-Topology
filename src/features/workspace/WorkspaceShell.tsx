import { useEffect, useState, type PointerEvent as ReactPointerEvent } from 'react'
import {
  inspectLocalElement,
  inspectRemoteElement,
  isTauriRuntime,
  type ElementMetadataResponse,
  type RemoteTargetInput,
} from '../../app/backend.ts'
import type { GStreamerRuntimeStatus, RuntimeEndpointStatus } from '../../app/status.ts'
import { ConnectionBadge } from '../../components/ConnectionBadge.tsx'
import { IconButton } from '../../components/IconButton.tsx'
import { InspectorPanel } from '../inspector/InspectorPanel.tsx'
import { GraphCanvas } from '../../graph/GraphCanvas.tsx'
import { DiagnosticsPanel } from './DiagnosticsPanel.tsx'
import { SourceTextPanel } from './SourceTextPanel.tsx'
import type {
  PipelineDocumentViewModel,
  PipelineDiagnostic,
  PipelineNodeViewModel,
  SourceSpan,
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

type SourceFocus = {
  id: string
  label: string
  span: SourceSpan
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

function clampPanelHeight(height: number) {
  const maxHeight = Math.round(window.innerHeight * 0.66)
  return Math.min(Math.max(height, 190), Math.max(240, maxHeight))
}

function panelLabel(panel: WorkspacePanel) {
  switch (panel) {
    case 'inspector':
      return 'Inspector'
    case 'source':
      return 'Pipeline Source'
    case 'diagnostics':
      return 'Parser Diagnostics'
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
  const [selectionRevision, setSelectionRevision] = useState(0)
  const [activePanel, setActivePanel] = useState<WorkspacePanel | null>(null)
  const [bottomPanelHeight, setBottomPanelHeight] = useState(320)
  const [sourceFocus, setSourceFocus] = useState<SourceFocus | null>(null)
  const [metadataByFactory, setMetadataByFactory] = useState<Record<string, MetadataEntry>>({})

  const selectedNode = findNode(document, selectedNodeId)
  const syntaxDiagnostics = document.diagnostics.filter(
    (diagnostic) => diagnostic.severity !== 'info',
  )
  const selectedMetadata = selectedNode
    ? metadataByFactory[selectedNode.factoryName]
    : undefined
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
  const runtimeModeStatus: RuntimeEndpointStatus = remoteTarget
    ? {
        ...gstreamerStatus.remote,
        host: remoteTarget.host,
        port: remoteTarget.port,
        message: gstreamerStatus.remote.message ?? 'Remote',
      }
    : {
        state: 'connected',
        message: 'Local',
      }

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

  function handleSelectNode(nodeId: string | null) {
    setSourceFocus(null)
    setSelectedNodeId(nodeId)
    setSelectionRevision((current) => current + 1)
  }

  function handleShowDiagnosticSource(diagnostic: PipelineDiagnostic) {
    if (!diagnostic.sourceSpan?.start && diagnostic.sourceSpan?.start !== 0) {
      return
    }

    setSourceFocus({
      id: diagnostic.id,
      label: diagnostic.message,
      span: diagnostic.sourceSpan,
    })
    setActivePanel('source')
    setSelectionRevision((current) => current + 1)
  }

  function handleResizeStart(event: ReactPointerEvent<HTMLDivElement>) {
    event.preventDefault()
    const startY = event.clientY
    const startHeight = bottomPanelHeight

    function handlePointerMove(moveEvent: PointerEvent) {
      setBottomPanelHeight(clampPanelHeight(startHeight + startY - moveEvent.clientY))
    }

    function handlePointerUp() {
      window.removeEventListener('pointermove', handlePointerMove)
      window.removeEventListener('pointerup', handlePointerUp)
    }

    window.addEventListener('pointermove', handlePointerMove)
    window.addEventListener('pointerup', handlePointerUp)
  }

  return (
    <main className="app-shell app-shell--workspace">
      <section className="workspace-shell">
        <header className="workspace-topbar panel">
          <div className="workspace-topbar__meta">
            <IconButton icon="arrowLeft" label="첫 화면으로 돌아가기" onClick={onBackHome} />
            <div className="workspace-topbar__title">
              <h1>{document.title}</h1>
            </div>
          </div>

          <div className="workspace-topbar__actions">
            <ConnectionBadge label="GStreamer API" status={apiStatus} />
            <ConnectionBadge
              label={remoteTarget ? 'Remote' : 'Local'}
              status={runtimeModeStatus}
            />
            <div className="workspace-topbar__panel-actions" aria-label="보조 패널">
              <IconButton
                active={activePanel === 'inspector'}
                icon="panelRight"
                label="Inspector 열기"
                onClick={() => togglePanel('inspector')}
              />
              <IconButton
                active={activePanel === 'source'}
                icon="fileText"
                label="Pipeline Source 열기"
                onClick={() => togglePanel('source')}
              />
              <IconButton
                active={activePanel === 'diagnostics'}
                badge={document.diagnostics.length || undefined}
                icon="diagnostics"
                label="Parser Diagnostics 열기"
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

        <div className={activePanel ? 'workspace-grid workspace-grid--bottom-open' : 'workspace-grid'}>
          <section className="workspace-main">
            <GraphCanvas
              document={document}
              selectedNodeId={selectedNodeId}
              onSelectNode={handleSelectNode}
            />
          </section>

          {activePanel ? (
            <section
              className="workspace-bottom-panel panel"
              style={{ height: bottomPanelHeight }}
            >
              <div
                aria-label="하단 패널 높이 조절"
                className="workspace-bottom-panel__resize"
                onPointerDown={handleResizeStart}
                role="separator"
                tabIndex={0}
              />
              <div className="workspace-bottom-panel__header">
                <div className="workspace-bottom-panel__tabs" role="tablist">
                  {(['inspector', 'source', 'diagnostics'] as WorkspacePanel[]).map((panel) => (
                    <button
                      aria-selected={activePanel === panel}
                      className={activePanel === panel ? 'is-active' : ''}
                      key={panel}
                      onClick={() => setActivePanel(panel)}
                      role="tab"
                      type="button"
                    >
                      {panelLabel(panel)}
                    </button>
                  ))}
                </div>
                <div className="workspace-bottom-panel__meta">
                  <span className="card-chip muted-chip">
                    {panelLabel(activePanel)}
                  </span>
                  <IconButton icon="close" label="하단 패널 닫기" onClick={() => setActivePanel(null)} />
                </div>
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
                  focusedSource={sourceFocus}
                  isOpen
                  selectedNode={selectedNode}
                  selectionRevision={selectionRevision}
                  onToggle={() => setActivePanel(null)}
                />
              ) : null}

              {activePanel === 'diagnostics' ? (
                <DiagnosticsPanel
                  diagnostics={document.diagnostics}
                  isOpen
                  onShowSource={handleShowDiagnosticSource}
                  onToggle={() => setActivePanel(null)}
                />
              ) : null}
            </section>
          ) : null}
        </div>
      </section>
    </main>
  )
}

export { WorkspaceShell }
