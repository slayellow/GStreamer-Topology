import { MarkerType, type Edge, type Node } from '@xyflow/react'
import type {
  PipelineEdgeViewModel,
  PipelineGraphViewModel,
  PipelineNodeTone,
  PipelineNodeViewModel,
  PipelinePortViewModel,
} from './types.ts'

type ReactFlowNodeParams = {
  graph: PipelineGraphViewModel
  portIndex?: NodePortIndex
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
  ports: TechnicalNodePort[]
  tags: string[]
  tone: PipelineNodeTone
  warningCount: number
}

type TechnicalFlowNode = Node<TechnicalNodeData, 'technical'>
type TechnicalNodePort = {
  edgeSourceNodeId: string
  edgeTargetNodeId: string
  id: string
  isConnectedToSelection: boolean
  label: string
  side: 'source' | 'target'
}

type IndexedNodePort = Omit<TechnicalNodePort, 'isConnectedToSelection'>
type NodePortIndex = Map<string, IndexedNodePort[]>

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

function sourceHandleId(edge: PipelineEdgeViewModel) {
  return `${edge.id}:source:${edge.sourcePort?.id ?? 'src'}`
}

function targetHandleId(edge: PipelineEdgeViewModel) {
  return `${edge.id}:target:${edge.targetPort?.id ?? 'sink'}`
}

function portLabel(port: PipelinePortViewModel | undefined, fallback: string) {
  if (!port) {
    return fallback
  }

  const name = port.name.trim()
  if (!name || name === 'src' || name === 'sink') {
    return fallback
  }

  return name
}

function buildNodePorts(
  portIndex: NodePortIndex,
  nodeId: string,
  selectedNodeId: string | null,
) {
  return (portIndex.get(nodeId) ?? []).map((port) => ({
    ...port,
    isConnectedToSelection: Boolean(
      selectedNodeId
        && (port.edgeSourceNodeId === selectedNodeId || port.edgeTargetNodeId === selectedNodeId),
    ),
  }))
}

function addIndexedPort(
  portIndex: NodePortIndex,
  nodeId: string,
  port: IndexedNodePort,
) {
  const ports = portIndex.get(nodeId) ?? []
  ports.push(port)
  portIndex.set(nodeId, ports)
}

function createNodePortIndex(graph: PipelineGraphViewModel): NodePortIndex {
  const nodeOrder = new Map(graph.nodes.map((node, index) => [node.id, index]))
  const portIndex: NodePortIndex = new Map()

  graph.edges.forEach((edge, index) => {
    const sourceId = sourceHandleId(edge)
    addIndexedPort(portIndex, edge.sourceNodeId, {
      edgeSourceNodeId: edge.sourceNodeId,
      edgeTargetNodeId: edge.targetNodeId,
      id: sourceId,
      label: portLabel(edge.sourcePort, `SRC ${index + 1}`),
      side: 'source',
    })

    const targetId = targetHandleId(edge)
    addIndexedPort(portIndex, edge.targetNodeId, {
      edgeSourceNodeId: edge.sourceNodeId,
      edgeTargetNodeId: edge.targetNodeId,
      id: targetId,
      label: portLabel(edge.targetPort, `SINK ${index + 1}`),
      side: 'target',
    })
  })

  for (const ports of portIndex.values()) {
    ports.sort((first, second) => {
      const firstOtherNode =
        first.side === 'source' ? first.edgeTargetNodeId : first.edgeSourceNodeId
      const secondOtherNode =
        second.side === 'source' ? second.edgeTargetNodeId : second.edgeSourceNodeId

      return (
        (nodeOrder.get(firstOtherNode) ?? 0) - (nodeOrder.get(secondOtherNode) ?? 0)
        || first.label.localeCompare(second.label)
        || first.id.localeCompare(second.id)
      )
    })
  }

  return portIndex
}

function toReactFlowNodes({
  graph,
  portIndex,
  searchValue,
  selectedNodeId,
}: ReactFlowNodeParams): TechnicalFlowNode[] {
  const indexedPorts = portIndex ?? createNodePortIndex(graph)

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
        ports: buildNodePorts(indexedPorts, node.id, selectedNodeId),
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
    const isMuted = Boolean(selectedNodeId && !isConnected)
    const label = edge.label?.trim()

    return {
      id: edge.id,
      source: edge.sourceNodeId,
      sourceHandle: sourceHandleId(edge),
      target: edge.targetNodeId,
      targetHandle: targetHandleId(edge),
      animated: false,
      type: 'smoothstep',
      ariaLabel: label ?? `${edge.sourceNodeId} to ${edge.targetNodeId}`,
      className: [
        'technical-edge',
        isConnected ? 'is-connected' : '',
        isMuted ? 'is-muted' : '',
      ].filter(Boolean).join(' '),
      interactionWidth: 22,
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
        strokeOpacity: isMuted ? 0.24 : 0.82,
        strokeWidth: isConnected ? 3 : 1.7,
      },
      labelStyle: {
        fill: isConnected ? 'var(--edge-active)' : 'var(--text-soft)',
        fontSize: label && label.length > 80 ? 12 : 11,
        fontWeight: 800,
        opacity: isMuted ? 0.28 : 1,
      },
      labelBgStyle: {
        fill: 'rgba(247, 250, 253, 0.96)',
        fillOpacity: isMuted ? 0.32 : 1,
        stroke: isConnected ? 'rgba(21, 141, 255, 0.28)' : 'rgba(23, 49, 82, 0.14)',
        strokeWidth: 1,
      },
      labelBgBorderRadius: 10,
      labelBgPadding: [10, 6] as [number, number],
      pathOptions: {
        borderRadius: 18,
        offset: 28,
      },
      zIndex: isConnected ? 1 : 0,
    }
  })
}

export { createNodePortIndex, toReactFlowEdges, toReactFlowNodes }
export type { NodePortIndex, TechnicalFlowNode, TechnicalNodeData, TechnicalNodePort }
