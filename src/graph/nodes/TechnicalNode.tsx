import { Handle, Position, type NodeProps } from '@xyflow/react'
import type { TechnicalFlowNode } from '../toReactFlow.ts'

function nodeKindLabel(kind: string) {
  switch (kind) {
    case 'element':
      return '요소'
    case 'virtual_group':
      return '가상 그룹'
    case 'caps':
      return '캡스'
    case 'unknown':
      return '알 수 없음'
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
      <div className="technical-node__eyebrow">
        <span>{nodeKindLabel(data.kind)}</span>
        {data.warningCount ? <span>경고 {data.warningCount}개</span> : null}
      </div>
      <strong>{data.label}</strong>
      <p>{data.factoryName}</p>
      <div className="technical-node__tags">
        {data.tags.slice(0, 2).map((tag) => (
          <span key={tag}>{tag}</span>
        ))}
      </div>
      <Handle className="technical-node__handle" position={Position.Right} type="source" />
    </div>
  )
}

export { TechnicalNode }
