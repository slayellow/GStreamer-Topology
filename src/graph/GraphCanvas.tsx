import { useEffect, useMemo, useState } from 'react'
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
import { TechnicalNode } from './nodes/TechnicalNode.tsx'
import { toReactFlowEdges, toReactFlowNodes, type TechnicalFlowNode } from './toReactFlow.ts'
import type { PipelineDocumentViewModel } from './types.ts'

type GraphCanvasProps = {
  document: PipelineDocumentViewModel
  selectedNodeId: string | null
  onSelectNode: (nodeId: string | null) => void
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

function GraphCanvas({
  document,
  selectedNodeId,
  onSelectNode,
}: GraphCanvasProps) {
  const [flow, setFlow] = useState<ReactFlowInstance<TechnicalFlowNode, Edge> | null>(null)
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

  useEffect(() => {
    if (!flow || !selectedNodeId) {
      return
    }

    const selectedNode = flow.getNode(selectedNodeId)

    if (!selectedNode) {
      return
    }

    flow.setCenter(selectedNode.position.x + 120, selectedNode.position.y + 56, {
      duration: 280,
      zoom: 0.92,
    })
  }, [flow, selectedNodeId])

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

  return (
    <section className="workspace-panel canvas-panel">
      <div className="graph-stage">
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
            <strong>토폴로지 캔버스</strong>
            <span>
              {selectedNode
                ? `선택됨: ${selectedNode.instanceName || selectedNode.factoryName}`
                : '노드를 클릭해 세부 정보를 확인하세요.'}
            </span>
          </Panel>
          <Panel className="graph-badge graph-badge--stats" position="top-right">
            <span>노드 {document.graph.nodes.length}개</span>
            <span>엣지 {document.graph.edges.length}개</span>
            <button className="graph-inline-action" onClick={handleResetLayout} type="button">
              {hasManualLayout ? '레이아웃 초기화' : '자동 배치'}
            </button>
          </Panel>
        </ReactFlow>

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
