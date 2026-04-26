import { MarkerType, type Edge, type Node } from '@xyflow/react'
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
const DEFAULT_NODE_DIMENSIONS = {
  height: 118,
  width: 220,
}

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
  return graph.nodes.map((node) => {
    const dimensions = node.dimensions ?? DEFAULT_NODE_DIMENSIONS

    return {
      id: node.id,
      type: 'technical',
      position: node.position,
      dragHandle: '.technical-node__drag-handle',
      draggable: true,
      height: dimensions.height,
      width: dimensions.width,
      style: {
        height: dimensions.height,
        width: dimensions.width,
      },
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
    }
  })
}

function toReactFlowEdges({
  graph,
  selectedNodeId,
}: ReactFlowEdgeParams): Edge[] {
  return graph.edges.map((edge) => {
    const isConnected =
      edge.sourceNodeId === selectedNodeId || edge.targetNodeId === selectedNodeId
    const label = edge.label?.trim()

    return {
      id: edge.id,
      source: edge.sourceNodeId,
      target: edge.targetNodeId,
      animated: false,
      ariaLabel: label ?? `${edge.sourceNodeId} to ${edge.targetNodeId}`,
      className: isConnected ? 'technical-edge is-connected' : 'technical-edge',
      interactionWidth: 18,
      label,
      markerEnd: {
        color: isConnected ? 'var(--edge-active)' : 'var(--edge-muted)',
        height: 18,
        type: MarkerType.ArrowClosed,
        width: 18,
      },
      style: {
        stroke: isConnected ? 'var(--edge-active)' : 'var(--edge-muted)',
        strokeLinecap: 'round',
        strokeWidth: isConnected ? 2.6 : 1.6,
      },
      labelStyle: {
        fill: isConnected ? 'var(--edge-active)' : 'var(--text-soft)',
        fontSize: label && label.length > 80 ? 12 : 11,
        fontWeight: 800,
      },
      labelBgStyle: {
        fill: 'rgba(247, 250, 253, 0.96)',
        fillOpacity: 1,
        stroke: isConnected ? 'rgba(21, 141, 255, 0.28)' : 'rgba(23, 49, 82, 0.14)',
        strokeWidth: 1,
      },
      labelBgBorderRadius: 10,
      labelBgPadding: [10, 6] as [number, number],
      zIndex: isConnected ? 4 : 1,
    }
  })
}

export { toReactFlowEdges, toReactFlowNodes }
export type { TechnicalFlowNode, TechnicalNodeData }
