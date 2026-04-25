import { useEffect, useState } from 'react'
import {
  Background,
  BackgroundVariant,
  Controls,
  MiniMap,
  Panel,
  ReactFlow,
  type Edge,
  type Node,
  type NodeMouseHandler,
  type ReactFlowInstance,
} from '@xyflow/react'
import { TechnicalNode } from './nodes/TechnicalNode.tsx'
import { toReactFlowEdges, toReactFlowNodes } from './toReactFlow.ts'
import type { PipelineDocumentViewModel } from './types.ts'

type GraphCanvasProps = {
  document: PipelineDocumentViewModel
  selectedNodeId: string | null
  onSelectNode: (nodeId: string | null) => void
}

const nodeTypes = {
  technical: TechnicalNode,
}

function GraphCanvas({
  document,
  selectedNodeId,
  onSelectNode,
}: GraphCanvasProps) {
  const [flow, setFlow] = useState<ReactFlowInstance<Node, Edge> | null>(null)
  const selectedNode = document.graph.nodes.find((node) => node.id === selectedNodeId) ?? null

  const nodes = toReactFlowNodes({
    graph: document.graph,
    searchValue: '',
    selectedNodeId,
  })
  const edges = toReactFlowEdges({
    graph: document.graph,
    selectedNodeId,
  })

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

    const selectedNode = nodes.find((node) => node.id === selectedNodeId)

    if (!selectedNode) {
      return
    }

    flow.setCenter(selectedNode.position.x + 120, selectedNode.position.y + 56, {
      duration: 280,
      zoom: 0.92,
    })
  }, [flow, nodes, selectedNodeId])

  const handleNodeClick: NodeMouseHandler<Node> = (_, node) => {
    onSelectNode(node.id)
  }

  return (
    <section className="workspace-panel canvas-panel">
      <div className="graph-stage">
        <ReactFlow
          defaultEdgeOptions={{ type: 'smoothstep' }}
          edges={edges}
          fitView
          maxZoom={1.4}
          minZoom={0.25}
          nodeTypes={nodeTypes}
          nodes={nodes}
          onInit={setFlow}
          onNodeClick={handleNodeClick}
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
            className="graph-minimap"
            maskColor="rgba(9, 18, 30, 0.12)"
            nodeBorderRadius={18}
            nodeColor="var(--minimap-node)"
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
