import { useEffect, useMemo, useRef, useState } from 'react'
import { toJpeg, toPng } from 'html-to-image'
import {
  applyNodeChanges,
  Background,
  BackgroundVariant,
  Controls,
  MiniMap,
  Panel,
  ReactFlow,
  type Edge,
  type NodeChange,
  type NodeMouseHandler,
  type ReactFlowInstance,
} from '@xyflow/react'
import { isTauriRuntime, saveExportFile } from '../app/backend.ts'
import { IconButton } from '../components/IconButton.tsx'
import { TechnicalNode } from './nodes/TechnicalNode.tsx'
import { toReactFlowEdges, toReactFlowNodes, type TechnicalFlowNode } from './toReactFlow.ts'
import type { PipelineDocumentViewModel } from './types.ts'

type GraphCanvasProps = {
  document: PipelineDocumentViewModel
  selectedNodeId: string | null
  onSelectNode: (nodeId: string | null) => void
}

type ExportFormat = 'jpg' | 'png'

type ExportStatus = {
  tone: 'error' | 'info' | 'success'
  message: string
}

const nodeTypes = {
  technical: TechnicalNode,
}

const minimapToneColors: Record<TechnicalFlowNode['data']['tone'], string> = {
  ai: '#2ec9d1',
  branch: '#7c8dff',
  merge: '#35b99f',
  output: '#64748b',
  source: '#158dff',
  unknown: '#94a3b8',
  utility: '#8b9cff',
}

function getMinimapNodeColor(node: TechnicalFlowNode) {
  return minimapToneColors[node.data.tone] ?? minimapToneColors.unknown
}

function getMinimapNodeStrokeColor(node: TechnicalFlowNode) {
  return node.data.isSelected ? '#ffffff' : 'rgba(2, 18, 36, 0.72)'
}

function graphSignature(document: PipelineDocumentViewModel) {
  return [
    document.graph.nodes.map((node) => node.id).join(','),
    document.graph.edges
      .map((edge) => (
        `${edge.sourceNodeId}:${edge.sourcePort?.name ?? 'src'}->${edge.targetNodeId}:${edge.targetPort?.name ?? 'sink'}`
      ))
      .join(','),
  ].join('|')
}

function storageKey(document: PipelineDocumentViewModel) {
  return `gst-topology-layout:${document.id}:${graphSignature(document)}`
}

type StoredPositions = Record<string, { x: number; y: number }>

function readStoredPositions(key: string): StoredPositions {
  const stored = localStorage.getItem(key)
  if (!stored) {
    return {}
  }

  try {
    return JSON.parse(stored) as StoredPositions
  } catch {
    return {}
  }
}

function extractPositions(nodes: TechnicalFlowNode[]) {
  return Object.fromEntries(
    nodes.map((node) => [node.id, node.position]),
  )
}

function savePositions(positions: StoredPositions, key: string) {
  localStorage.setItem(key, JSON.stringify(positions))
}

function safeFileStem(value: string) {
  return value
    .trim()
    .replace(/\.[^/.]+$/, '')
    .replace(/[^a-z0-9가-힣_-]+/gi, '-')
    .replace(/^-+|-+$/g, '')
    || 'gstreamer-topology'
}

function splitDataUrl(dataUrl: string) {
  const commaIndex = dataUrl.indexOf(',')
  if (commaIndex === -1) {
    return { metadata: '', payload: dataUrl }
  }

  return {
    metadata: dataUrl.slice(0, commaIndex),
    payload: dataUrl.slice(commaIndex + 1),
  }
}

function downloadFromBrowser(dataUrl: string, fileName: string) {
  const link = document.createElement('a')
  link.download = fileName
  link.href = dataUrl
  link.rel = 'noopener'
  document.body.appendChild(link)
  link.click()
  link.remove()
}

function shouldExportNode(node: Node) {
  if (!(node instanceof Element)) {
    return true
  }

  return !(
    node.getAttribute('data-export-exclude') === 'true' ||
    node.classList.contains('react-flow__controls') ||
    node.classList.contains('react-flow__minimap')
  )
}

function GraphCanvas({
  document,
  selectedNodeId,
  onSelectNode,
}: GraphCanvasProps) {
  const [flow, setFlow] = useState<ReactFlowInstance<TechnicalFlowNode, Edge> | null>(null)
  const stageRef = useRef<HTMLDivElement | null>(null)
  const key = storageKey(document)
  const selectedNode = document.graph.nodes.find((node) => node.id === selectedNodeId) ?? null
  const renderedNodes = useMemo(() => toReactFlowNodes({
    graph: document.graph,
    searchValue: '',
    selectedNodeId,
  }), [document.graph, selectedNodeId])
  const [manualPositions, setManualPositions] = useState<StoredPositions>(() =>
    readStoredPositions(key),
  )
  const [exportMenuOpen, setExportMenuOpen] = useState(false)
  const [exportStatus, setExportStatus] = useState<ExportStatus | null>(null)
  const nodes = useMemo(
    () =>
      renderedNodes.map((node) => ({
        ...node,
        position: manualPositions[node.id] ?? node.position,
      })),
    [manualPositions, renderedNodes],
  )
  const hasManualLayout = Object.keys(manualPositions).length > 0
  const edges = useMemo(() => toReactFlowEdges({
    graph: document.graph,
    selectedNodeId,
  }), [document.graph, selectedNodeId])

  useEffect(() => {
    if (!flow || !nodes.length) {
      return
    }

    flow.fitView({
      duration: 360,
      padding: 0.18,
    })
  }, [document.id, flow, nodes.length])

  const handleNodeClick: NodeMouseHandler<TechnicalFlowNode> = (_, node) => {
    onSelectNode(node.id)
  }
  const handleNodesChange = (changes: NodeChange<TechnicalFlowNode>[]) => {
    const positionChanges = changes.filter(
      (change) => change.type === 'position' && change.position,
    )

    if (!positionChanges.length) {
      return
    }

    const nextNodes = applyNodeChanges(positionChanges, nodes)
    setManualPositions(extractPositions(nextNodes))
  }
  const handleNodeDragStop: NodeMouseHandler<TechnicalFlowNode> = () => {
    const currentNodes = (flow?.getNodes() as TechnicalFlowNode[] | undefined) ?? nodes
    const positions = extractPositions(currentNodes)
    setManualPositions(positions)
    savePositions(positions, key)
  }
  const handleResetLayout = () => {
    localStorage.removeItem(key)
    setManualPositions({})
    flow?.fitView({
      duration: 320,
      padding: 0.18,
    })
  }
  const handleExport = async (format: ExportFormat) => {
    const stage = stageRef.current
    if (!stage) {
      setExportStatus({
        tone: 'error',
        message: 'Export failed: canvas is not ready.',
      })
      return
    }

    setExportMenuOpen(false)
    setExportStatus({
      tone: 'info',
      message: `Exporting ${format.toUpperCase()}...`,
    })

    try {
      const fileName = `${safeFileStem(document.title)}.${format}`
      const options = {
        backgroundColor: '#eff6fd',
        cacheBust: true,
        filter: shouldExportNode,
        pixelRatio: 2,
      }
      const dataUrl =
        format === 'png'
          ? await toPng(stage, options)
          : await toJpeg(stage, { ...options, quality: 0.94 })

      if (isTauriRuntime()) {
        const defaultPath = fileName
        const path = window.prompt(
          '저장할 파일 경로를 입력하세요. 파일 이름만 입력하면 앱 실행 위치에 저장됩니다.',
          defaultPath,
        )

        if (path === null) {
          setExportStatus({ tone: 'info', message: 'Save canceled.' })
          return
        }

        const savedPath = await saveExportFile(
          path,
          splitDataUrl(dataUrl).payload,
        )
        setExportStatus({
          tone: 'success',
          message: savedPath ? `Saved: ${savedPath}` : 'Save canceled.',
        })
        return
      }

      downloadFromBrowser(dataUrl, fileName)
      setExportStatus({
        tone: 'success',
        message: `Downloaded ${fileName}.`,
      })
    } catch (error) {
      console.error(error)
      setExportStatus({
        tone: 'error',
        message: `Export failed: ${error instanceof Error ? error.message : 'unknown error'}`,
      })
    }
  }

  return (
    <section className="workspace-panel canvas-panel">
      <div className="graph-stage" ref={stageRef}>
        <ReactFlow<TechnicalFlowNode, Edge>
          defaultEdgeOptions={{ type: 'smoothstep' }}
          edges={edges}
          fitView
          maxZoom={1.4}
          minZoom={0.25}
          nodeTypes={nodeTypes}
          nodes={nodes}
          onInit={setFlow}
          onNodeClick={handleNodeClick}
          onNodeDragStop={handleNodeDragStop}
          onNodesChange={handleNodesChange}
          onPaneClick={() => onSelectNode(null)}
          onlyRenderVisibleElements
          proOptions={{ hideAttribution: true }}
        >
          <Background
            color="var(--canvas-dot)"
            gap={22}
            size={1.2}
            variant={BackgroundVariant.Dots}
          />
          <MiniMap
            bgColor="rgba(229, 238, 248, 0.96)"
            className="graph-minimap"
            maskColor="rgba(9, 23, 42, 0.24)"
            maskStrokeColor="rgba(21, 88, 155, 0.72)"
            maskStrokeWidth={2}
            nodeBorderRadius={8}
            nodeColor={getMinimapNodeColor}
            nodeStrokeColor={getMinimapNodeStrokeColor}
            nodeStrokeWidth={2.4}
            pannable
            zoomable
          />
          <Controls position="bottom-left" showInteractive={false} />
          <Panel className="graph-badge" position="top-left">
            <strong>Topology Canvas</strong>
            <span>
              {selectedNode
                ? `Selected: ${selectedNode.instanceName || selectedNode.factoryName}`
                : 'Select a node for details.'}
            </span>
          </Panel>
          <Panel className="graph-badge graph-badge--stats" position="top-right">
            <span>Nodes {document.graph.nodes.length}</span>
            <span>Edges {document.graph.edges.length}</span>
            <button className="graph-inline-action" onClick={handleResetLayout} type="button">
              {hasManualLayout ? 'Reset Layout' : 'Auto Layout'}
            </button>
            <div className="graph-export" data-export-exclude="true">
              <IconButton
                active={exportMenuOpen}
                icon="download"
                label="Export topology"
                onClick={() => setExportMenuOpen((current) => !current)}
              />
              {exportMenuOpen ? (
                <div className="graph-export__menu">
                  {(['png', 'jpg'] as ExportFormat[]).map((format) => (
                    <button
                      key={format}
                      onClick={() => void handleExport(format)}
                      type="button"
                    >
                      {format.toUpperCase()}
                    </button>
                  ))}
                </div>
              ) : null}
            </div>
          </Panel>
        </ReactFlow>

        {exportStatus ? (
          <div
            className={`graph-export-status graph-export-status--${exportStatus.tone}`}
            data-export-exclude="true"
            role="status"
          >
            {exportStatus.message}
          </div>
        ) : null}

        {!document.graph.nodes.length ? (
          <div className="graph-empty-state">
            <span className="card-chip">그래프 없음</span>
            <p>
              이번 가져오기로는 그래프 노드가 생성되지 않았습니다. 진단 패널을
              확인한 뒤 다른 로컬 파일이나 붙여넣은 파이프라인을 시도하세요.
            </p>
          </div>
        ) : null}
      </div>
    </section>
  )
}

export { GraphCanvas }
