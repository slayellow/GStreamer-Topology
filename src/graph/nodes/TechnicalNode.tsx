import { Handle, Position, type NodeProps } from '@xyflow/react'
import type { TechnicalFlowNode } from '../toReactFlow.ts'

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

function TechnicalNode({ data }: NodeProps<TechnicalFlowNode>) {
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
      <Handle className="technical-node__handle" position={Position.Left} type="target" />
      <span aria-hidden className="technical-node__port-dot technical-node__port-dot--sink" />
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
      <span aria-hidden className="technical-node__port-dot technical-node__port-dot--src" />
      <Handle className="technical-node__handle" position={Position.Right} type="source" />
    </div>
  )
}

export { TechnicalNode }
