import type { Edge, Node } from '@xyflow/react'
import type {
  PipelineGraphViewModel,
  PipelineNodeTone,
  PipelineNodeViewModel,
} from './types.ts'

type ReactFlowNodeParams = {
  graph: PipelineGraphViewModel
  searchValue: string
  selectedNodeId: string | null
}

type ReactFlowEdgeParams = {
  graph: PipelineGraphViewModel
  selectedNodeId: string | null
}

type TechnicalNodeData = {
  factoryName: string
  kind: string
  isSearchMatch: boolean
  isSelected: boolean
  label: string
  tags: string[]
  tone: PipelineNodeTone
  warningCount: number
}

type TechnicalFlowNode = Node<TechnicalNodeData, 'technical'>

function matchesSearch(node: PipelineNodeViewModel, searchValue: string) {
  const normalizedQuery = searchValue.trim().toLowerCase()

  if (!normalizedQuery) {
    return false
  }

  return [node.instanceName, node.factoryName, node.tags.join(' '), node.description]
    .filter(Boolean)
    .join(' ')
    .toLowerCase()
    .includes(normalizedQuery)
}

function toReactFlowNodes({
  graph,
  searchValue,
  selectedNodeId,
}: ReactFlowNodeParams): TechnicalFlowNode[] {
  return graph.nodes.map((node) => ({
    id: node.id,
    type: 'technical',
    position: node.position,
    draggable: false,
    data: {
      factoryName: node.factoryName,
      kind: node.kind,
      isSearchMatch: matchesSearch(node, searchValue),
      isSelected: node.id === selectedNodeId,
      label: node.instanceName || node.factoryName,
      tags: node.tags,
      tone: node.tone,
      warningCount: node.warnings.length,
    },
  }))
}

function toReactFlowEdges({
  graph,
  selectedNodeId,
}: ReactFlowEdgeParams): Edge[] {
  return graph.edges.map((edge) => {
    const isConnected =
      edge.sourceNodeId === selectedNodeId || edge.targetNodeId === selectedNodeId

    return {
      id: edge.id,
      source: edge.sourceNodeId,
      target: edge.targetNodeId,
      animated: isConnected,
      label: edge.label,
      style: {
        stroke: isConnected ? 'var(--edge-active)' : 'var(--edge-muted)',
        strokeWidth: isConnected ? 2.4 : 1.4,
      },
      labelStyle: {
        fill: 'var(--text-soft)',
        fontSize: 12,
        fontWeight: 600,
      },
      labelBgStyle: {
        fill: 'rgba(245, 247, 251, 0.92)',
        fillOpacity: 1,
      },
    }
  })
}

export { toReactFlowEdges, toReactFlowNodes }
export type { TechnicalFlowNode, TechnicalNodeData }
