import { memo } from 'react'
import { Handle, Position, type NodeProps } from '@xyflow/react'
import type { TechnicalFlowNode, TechnicalNodePort } from '../toReactFlow.ts'

function nodeKindLabel(kind: string) {
  switch (kind) {
    case 'element':
      return 'Element'
    case 'virtual_group':
      return 'Virtual Group'
    case 'caps':
      return 'Caps'
    case 'unknown':
      return 'Unknown'
    default:
      return kind
  }
}

function portTop(index: number, total: number) {
  if (total <= 1) {
    return '50%'
  }

  return `${18 + (64 * index) / (total - 1)}%`
}

function renderPort(port: TechnicalNodePort, index: number, total: number, isSelected: boolean) {
  const isSource = port.side === 'source'
  const top = portTop(index, total)
  const position = isSource ? Position.Right : Position.Left
  const type = isSource ? 'source' : 'target'
  const sideClass = isSource ? 'source' : 'sink'
  const stateClass = port.isConnectedToSelection ? 'is-connected' : ''

  return (
    <span
      aria-hidden
      className={[
        'technical-node__port',
        `technical-node__port--${sideClass}`,
        stateClass,
        isSelected ? 'is-node-selected' : '',
      ].filter(Boolean).join(' ')}
      key={port.id}
      style={{ top }}
    >
      <Handle
        className="technical-node__handle"
        id={port.id}
        position={position}
        type={type}
      />
    </span>
  )
}

function TechnicalNode({ data }: NodeProps<TechnicalFlowNode>) {
  const sourcePorts = data.ports.filter((port) => port.side === 'source')
  const sinkPorts = data.ports.filter((port) => port.side === 'target')

  return (
    <div
      className={[
        'technical-node',
        `tone-${data.tone}`,
        data.isSelected ? 'is-selected' : '',
        data.isSearchMatch ? 'is-match' : '',
      ]
        .filter(Boolean)
        .join(' ')}
    >
      <span className="technical-node__port-rail technical-node__port-rail--sink">
        {sinkPorts.map((port, index) =>
          renderPort(port, index, sinkPorts.length, data.isSelected),
        )}
      </span>
      <div className="technical-node__eyebrow">
        <span>{nodeKindLabel(data.kind)}</span>
        <span className="technical-node__drag-handle" title="위치 이동">
          ::
        </span>
        {data.warningCount ? <span>{data.warningCount} warnings</span> : null}
      </div>
      <strong>{data.label}</strong>
      <p>{data.factoryName}</p>
      <div className="technical-node__tags">
        {data.tags.slice(0, 2).map((tag) => (
          <span key={tag}>{tag}</span>
        ))}
      </div>
      <span className="technical-node__port-rail technical-node__port-rail--source">
        {sourcePorts.map((port, index) =>
          renderPort(port, index, sourcePorts.length, data.isSelected),
        )}
      </span>
    </div>
  )
}

function portsEqual(first: TechnicalNodePort[], second: TechnicalNodePort[]) {
  if (first.length !== second.length) {
    return false
  }

  return first.every((port, index) => {
    const other = second[index]
    return (
      port.id === other.id
      && port.isConnectedToSelection === other.isConnectedToSelection
      && port.label === other.label
      && port.side === other.side
    )
  })
}

const MemoizedTechnicalNode = memo(TechnicalNode, (previous, next) => (
  previous.data.factoryName === next.data.factoryName
  && previous.data.kind === next.data.kind
  && previous.data.isSearchMatch === next.data.isSearchMatch
  && previous.data.isSelected === next.data.isSelected
  && previous.data.label === next.data.label
  && previous.data.tone === next.data.tone
  && previous.data.warningCount === next.data.warningCount
  && previous.data.tags.join('|') === next.data.tags.join('|')
  && portsEqual(previous.data.ports, next.data.ports)
))

export { MemoizedTechnicalNode as TechnicalNode }
