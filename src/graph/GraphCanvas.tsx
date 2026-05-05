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
  type Node as FlowNode,
  type ReactFlowInstance,
} from '@xyflow/react'
import {
  isTauriRuntime,
  saveExportFile,
  suggestExportFilePath,
} from '../app/backend.ts'
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

type ExportDraft = {
  fileName: string
  format: ExportFormat
  isLoadingPath: boolean
  message?: string
  path: string
}

type GraphBounds = {
  height: number
  maxX: number
  maxY: number
  minX: number
  minY: number
  width: number
}

const nodeTypes = {
  technical: TechnicalNode,
}

const EXPORT_PADDING = 180
const MAX_EXPORT_DIMENSION = 14000
const MAX_EXPORT_PIXELS = 42_000_000

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

function shouldExportNode(node: globalThis.Node) {
  if (!(node instanceof Element)) {
    return true
  }

  return !(
    node.getAttribute('data-export-exclude') === 'true' ||
    node.classList.contains('graph-badge') ||
    node.classList.contains('graph-export-status') ||
    node.classList.contains('react-flow__controls') ||
    node.classList.contains('react-flow__minimap')
  )
}

function nextAnimationFrame() {
  return new Promise<void>((resolve) => {
    window.requestAnimationFrame(() => resolve())
  })
}

function nodeDimension(node: FlowNode) {
  const styleWidth = typeof node.style?.width === 'number' ? node.style.width : undefined
  const styleHeight = typeof node.style?.height === 'number' ? node.style.height : undefined

  return {
    height: styleHeight ?? node.height ?? 118,
    width: styleWidth ?? node.width ?? 220,
  }
}

function includeEdgeLabelBounds(bounds: GraphBounds, nodes: FlowNode[], edges: Edge[]) {
  const nodeById = new Map(nodes.map((node) => [node.id, node]))

  return edges.reduce((current, edge) => {
    if (typeof edge.label !== 'string' || !edge.label.trim()) {
      return current
    }

    const source = nodeById.get(edge.source)
    const target = nodeById.get(edge.target)
    if (!source || !target) {
      return current
    }

    const sourceDimensions = nodeDimension(source)
    const targetDimensions = nodeDimension(target)
    const sourceCenter = {
      x: source.position.x + sourceDimensions.width / 2,
      y: source.position.y + sourceDimensions.height / 2,
    }
    const targetCenter = {
      x: target.position.x + targetDimensions.width / 2,
      y: target.position.y + targetDimensions.height / 2,
    }
    const labelCenter = {
      x: (sourceCenter.x + targetCenter.x) / 2,
      y: (sourceCenter.y + targetCenter.y) / 2,
    }
    const labelWidth = Math.max(120, edge.label.length * 7.6 + 56)
    const labelHeight = 44
    const nextMinX = Math.min(current.minX, labelCenter.x - labelWidth / 2)
    const nextMaxX = Math.max(current.maxX, labelCenter.x + labelWidth / 2)
    const nextMinY = Math.min(current.minY, labelCenter.y - labelHeight / 2)
    const nextMaxY = Math.max(current.maxY, labelCenter.y + labelHeight / 2)

    return {
      ...current,
      height: nextMaxY - nextMinY,
      maxX: nextMaxX,
      maxY: nextMaxY,
      minX: nextMinX,
      minY: nextMinY,
      width: nextMaxX - nextMinX,
    }
  }, bounds)
}

function getGraphBounds(nodes: FlowNode[], edges: Edge[]): GraphBounds | null {
  if (!nodes.length) {
    return null
  }

  const initial = {
    maxX: Number.NEGATIVE_INFINITY,
    maxY: Number.NEGATIVE_INFINITY,
    minX: Number.POSITIVE_INFINITY,
    minY: Number.POSITIVE_INFINITY,
  }

  const bounds = nodes.reduce((current, node) => {
    const dimensions = nodeDimension(node)
    return {
      maxX: Math.max(current.maxX, node.position.x + dimensions.width),
      maxY: Math.max(current.maxY, node.position.y + dimensions.height),
      minX: Math.min(current.minX, node.position.x),
      minY: Math.min(current.minY, node.position.y),
    }
  }, initial)

  return includeEdgeLabelBounds({
    ...bounds,
    height: bounds.maxY - bounds.minY,
    width: bounds.maxX - bounds.minX,
  }, nodes, edges)
}

function exportScale(width: number, height: number) {
  const dimensionScale = Math.min(
    MAX_EXPORT_DIMENSION / Math.max(width, 1),
    MAX_EXPORT_DIMENSION / Math.max(height, 1),
  )
  const areaScale = Math.sqrt(MAX_EXPORT_PIXELS / Math.max(width * height, 1))

  return Math.min(1, dimensionScale, areaScale)
}

async function captureFullTopologyImage({
  format,
  edges,
  nodes,
  stage,
}: {
  format: ExportFormat
  edges: Edge[]
  nodes: TechnicalFlowNode[]
  stage: HTMLDivElement
}) {
  const viewport = stage.querySelector<HTMLElement>('.react-flow__viewport')
  const flowRoot = stage.querySelector<HTMLElement>('.react-flow')
  const bounds = getGraphBounds(nodes, edges)

  if (!viewport || !flowRoot || !bounds) {
    throw new Error('full topology export target is not ready')
  }

  const rawWidth = bounds.width + EXPORT_PADDING * 2
  const rawHeight = bounds.height + EXPORT_PADDING * 2
  const scale = exportScale(rawWidth, rawHeight)
  const exportWidth = Math.ceil(rawWidth * scale)
  const exportHeight = Math.ceil(rawHeight * scale)
  const translateX = (EXPORT_PADDING - bounds.minX) * scale
  const translateY = (EXPORT_PADDING - bounds.minY) * scale

  const previousStageStyle = {
    height: stage.style.height,
    overflow: stage.style.overflow,
    width: stage.style.width,
  }
  const previousFlowStyle = {
    height: flowRoot.style.height,
    width: flowRoot.style.width,
  }
  const previousViewportStyle = {
    transform: viewport.style.transform,
    transformOrigin: viewport.style.transformOrigin,
  }

  try {
    stage.classList.add('is-exporting-full')
    stage.style.width = `${exportWidth}px`
    stage.style.height = `${exportHeight}px`
    stage.style.overflow = 'visible'
    flowRoot.style.width = `${exportWidth}px`
    flowRoot.style.height = `${exportHeight}px`
    viewport.style.transform = `translate(${translateX}px, ${translateY}px) scale(${scale})`
    viewport.style.transformOrigin = '0 0'

    await nextAnimationFrame()

    const options = {
      backgroundColor: '#eff6fd',
      cacheBust: true,
      filter: shouldExportNode,
      height: exportHeight,
      pixelRatio: 1,
      style: {
        height: `${exportHeight}px`,
        overflow: 'visible',
        width: `${exportWidth}px`,
      },
      width: exportWidth,
    }

    return format === 'png'
      ? toPng(stage, options)
      : toJpeg(stage, { ...options, quality: 0.94 })
  } finally {
    stage.classList.remove('is-exporting-full')
    stage.style.width = previousStageStyle.width
    stage.style.height = previousStageStyle.height
    stage.style.overflow = previousStageStyle.overflow
    flowRoot.style.width = previousFlowStyle.width
    flowRoot.style.height = previousFlowStyle.height
    viewport.style.transform = previousViewportStyle.transform
    viewport.style.transformOrigin = previousViewportStyle.transformOrigin
  }
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
  const [exportDraft, setExportDraft] = useState<ExportDraft | null>(null)
  const [isExportingFullGraph, setIsExportingFullGraph] = useState(false)
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
  const handleFocusSelected = () => {
    if (!flow || !selectedNodeId) {
      return
    }

    const node = nodes.find((candidate) => candidate.id === selectedNodeId)
    if (!node) {
      return
    }

    const width =
      typeof node.style?.width === 'number' ? node.style.width : node.width ?? 220
    const height =
      typeof node.style?.height === 'number' ? node.style.height : node.height ?? 118
    const currentZoom = flow.getZoom()
    const nextZoom = Math.min(Math.max(currentZoom, 0.58), 1.02)

    flow.setCenter(
      node.position.x + width / 2,
      node.position.y + height / 2,
      {
        duration: 260,
        zoom: nextZoom,
      },
    )
  }
  const handleExport = async (format: ExportFormat, explicitPath?: string) => {
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
      message: `Exporting full topology ${format.toUpperCase()}...`,
    })

    try {
      const fileName = `${safeFileStem(document.title)}.${format}`
      setIsExportingFullGraph(true)
      await nextAnimationFrame()
      await nextAnimationFrame()
      const dataUrl = await captureFullTopologyImage({
        edges,
        format,
        nodes,
        stage,
      })

      if (isTauriRuntime()) {
        const targetPath = explicitPath?.trim() ?? ''
        if (!targetPath) {
          setExportStatus({
            tone: 'info',
            message: 'Save canceled.',
          })
          return
        }

        const savedPath = await saveExportFile(targetPath, splitDataUrl(dataUrl).payload)
        if (!savedPath) {
          setExportStatus({
            tone: 'info',
            message: 'Save canceled.',
          })
          return
        }

        setExportStatus({
          tone: 'success',
          message: `Saved: ${savedPath}`,
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
    } finally {
      setIsExportingFullGraph(false)
    }
  }
  const handleOpenExportDraft = async (format: ExportFormat) => {
    const fileName = `${safeFileStem(document.title)}.${format}`
    setExportMenuOpen(false)

    if (!isTauriRuntime()) {
      await handleExport(format)
      return
    }

    setExportDraft({
      fileName,
      format,
      isLoadingPath: true,
      message: undefined,
      path: '',
    })

    try {
      const path = await suggestExportFilePath(fileName)
      setExportDraft((current) =>
        current?.format === format
          ? {
              ...current,
              isLoadingPath: false,
              message: undefined,
              path,
            }
          : current,
      )
    } catch (error) {
      console.error(error)
      setExportDraft((current) =>
        current?.format === format
          ? {
              ...current,
              isLoadingPath: false,
              message: '기본 저장 경로를 만들지 못했습니다. 직접 전체 경로를 입력해 주세요.',
              path: fileName,
            }
          : current,
      )
    }
  }
  const handleSaveExportDraft = () => {
    if (!exportDraft) {
      return
    }

    if (!exportDraft.path.trim()) {
      setExportDraft((current) =>
        current
          ? {
              ...current,
              message: '저장할 파일 경로를 입력해 주세요.',
            }
          : current,
      )
      return
    }

    const draft = exportDraft
    setExportDraft(null)
    void handleExport(draft.format, draft.path)
  }
  const handleCancelExportDraft = () => {
    setExportDraft(null)
    setExportStatus({
      tone: 'info',
      message: 'Save canceled.',
    })
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
          onlyRenderVisibleElements={!isExportingFullGraph}
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
          <Panel className="graph-badge" data-export-exclude="true" position="top-left">
            <strong>Topology Canvas</strong>
            <span>
              {selectedNode
                ? `Selected: ${selectedNode.instanceName || selectedNode.factoryName}`
                : 'Select a node for details.'}
            </span>
          </Panel>
          <Panel
            className="graph-badge graph-badge--stats"
            data-export-exclude="true"
            position="top-right"
          >
            <span>Nodes {document.graph.nodes.length}</span>
            <span>Edges {document.graph.edges.length}</span>
            <button
              className="graph-inline-action"
              disabled={!selectedNodeId}
              onClick={handleFocusSelected}
              type="button"
            >
              Focus Selected
            </button>
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
                      onClick={() => void handleOpenExportDraft(format)}
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

      {exportDraft ? (
        <div className="graph-export-modal" role="dialog" aria-modal="true">
          <div className="graph-export-modal__card">
            <div>
              <span className="field-label">Export {exportDraft.format.toUpperCase()}</span>
              <h3>저장 경로 확인</h3>
              <p>
                전체 토폴로지 영역을 이미지로 저장합니다. 기본값은 Downloads 아래의
                `GStreamer Topology Exports` 폴더입니다. 다른 위치를 원하면 전체
                파일 경로를 수정해 주세요.
              </p>
            </div>
            <label>
              <span>파일 경로</span>
              <input
                autoFocus
                disabled={exportDraft.isLoadingPath}
                onChange={(event) =>
                  setExportDraft((current) =>
                    current
                      ? {
                          ...current,
                          message: undefined,
                          path: event.target.value,
                        }
                      : current,
                  )
                }
                placeholder={exportDraft.fileName}
                value={exportDraft.path}
              />
            </label>
            {exportDraft.message ? (
              <p className="graph-export-modal__message">{exportDraft.message}</p>
            ) : null}
            <div className="graph-export-modal__actions">
              <button onClick={handleCancelExportDraft} type="button">
                취소
              </button>
              <button
                disabled={exportDraft.isLoadingPath}
                onClick={handleSaveExportDraft}
                type="button"
              >
                저장
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </section>
  )
}

export { GraphCanvas }
