import type { ElementMetadataResponse } from '../../app/backend.ts'
import type {
  PipelineDiagnostic,
  PipelineDocumentViewModel,
  PipelineNodeViewModel,
  PipelineProperty,
} from '../../graph/types.ts'

type InspectorPanelProps = {
  document: PipelineDocumentViewModel
  metadata?: {
    data?: ElementMetadataResponse
    message?: string
    status: 'loading' | 'ready' | 'unavailable'
  }
  selectedNode: PipelineNodeViewModel | null
}

type ConnectionSummary = {
  incoming: PipelineNodeViewModel[]
  outgoing: PipelineNodeViewModel[]
}

function diagnosticSeverityLabel(severity: PipelineDiagnostic['severity']) {
  switch (severity) {
    case 'info':
      return '정보'
    case 'warning':
      return '경고'
    case 'error':
      return '오류'
  }
}

function nodeKindLabel(kind: PipelineNodeViewModel['kind']) {
  switch (kind) {
    case 'element':
      return '요소'
    case 'virtual_group':
      return '가상 그룹'
    case 'caps':
      return '캡스'
    case 'unknown':
      return '알 수 없음'
  }
}

function findConnections(
  document: PipelineDocumentViewModel,
  node: PipelineNodeViewModel,
): ConnectionSummary {
  const incomingIds = document.graph.edges
    .filter((edge) => edge.targetNodeId === node.id)
    .map((edge) => edge.sourceNodeId)
  const outgoingIds = document.graph.edges
    .filter((edge) => edge.sourceNodeId === node.id)
    .map((edge) => edge.targetNodeId)

  return {
    incoming: document.graph.nodes.filter((candidate) =>
      incomingIds.includes(candidate.id),
    ),
    outgoing: document.graph.nodes.filter((candidate) =>
      outgoingIds.includes(candidate.id),
    ),
  }
}

function renderProperty(property: PipelineProperty) {
  return (
    <li key={`${property.key}-${property.value}`}>
      <span>{property.key}</span>
      <code>{property.value}</code>
    </li>
  )
}

function renderDiagnostic(diagnostic: PipelineDiagnostic) {
  return (
    <li key={diagnostic.id} className={`inspector-diagnostic severity-${diagnostic.severity}`}>
      <span>{diagnosticSeverityLabel(diagnostic.severity)}</span>
      <p>{diagnostic.message}</p>
    </li>
  )
}

function metadataAuthorityLabel(metadata: ElementMetadataResponse) {
  return metadata.authority === 'remote' ? '원격 GStreamer' : '로컬 GStreamer'
}

function metadataPropertySummary(property: ElementMetadataResponse['properties'][number]) {
  return [
    property.value_type ? `Type: ${property.value_type}` : null,
    property.default_value ? `Default: ${property.default_value}` : null,
    property.current_value ? `Current: ${property.current_value}` : null,
  ].filter((value): value is string => Boolean(value))
}

function InspectorPanel({ document, metadata, selectedNode }: InspectorPanelProps) {
  if (!selectedNode) {
    return (
      <aside className="workspace-panel inspector-panel">
        <div className="section-heading">
          <div className="eyebrow">인스펙터</div>
          <h2>캔버스에서 노드를 선택하세요</h2>
        </div>
        <p className="muted-copy">
          토폴로지에서 요소를 선택하면 속성, 연결, 경고, 원본 위치를 확인할 수
          있습니다.
        </p>
        <div className="inspector-empty-state">
          <span className="card-chip">선택 대기 중</span>
          <p>노드 세부 정보, 링크, 경고, 원본 범위 정보가 여기에 표시됩니다.</p>
        </div>
      </aside>
    )
  }

  const connections = findConnections(document, selectedNode)
  const metadataProperties = metadata?.data?.properties ?? []

  return (
    <aside className="workspace-panel inspector-panel">
      <div className="section-heading">
        <div className="eyebrow">인스펙터</div>
        <h2>{selectedNode.instanceName || selectedNode.factoryName}</h2>
      </div>

      <div className="inspector-card">
        <div className="inspector-card__header">
          <span className={`card-chip tone-${selectedNode.tone}`}>
            {nodeKindLabel(selectedNode.kind)}
          </span>
          <span className="card-chip">{selectedNode.factoryName}</span>
        </div>
        <p>{selectedNode.description}</p>
      </div>

      <section className="inspector-section">
        <h3>원본 범위</h3>
        {selectedNode.sourceSpan ? (
          <p>
            줄 {selectedNode.sourceSpan.lineStart}-{selectedNode.sourceSpan.lineEnd}
          </p>
        ) : (
          <p className="muted-copy">이 노드에는 원본 줄 매핑이 없습니다.</p>
        )}
      </section>

      <section className="inspector-section">
        <h3>속성</h3>
        {selectedNode.properties.length ? (
          <ul className="inspector-property-list">
            {selectedNode.properties.map(renderProperty)}
          </ul>
        ) : (
          <p className="muted-copy">아직 캡처된 명시적 속성이 없습니다.</p>
        )}
      </section>

      <section className="inspector-section">
        <h3>Element 내부 정보</h3>
        {!metadata || metadata.status === 'loading' ? (
          <div className="metadata-card">
            <span className="card-chip muted-chip">조회 중</span>
            <p>선택한 Element의 GStreamer 메타데이터를 확인하고 있습니다.</p>
          </div>
        ) : metadata.status === 'unavailable' || !metadata.data?.available ? (
          <div className="metadata-card">
            <span className="card-chip muted-chip">텍스트 파서 기준</span>
            <p>
              {metadata.message ??
                '이 환경에서 해당 Element의 GStreamer 내부 정보를 가져오지 못했습니다.'}
            </p>
          </div>
        ) : (
          <div className="metadata-card">
            <div className="inspector-card__header">
              <span className="card-chip">{metadataAuthorityLabel(metadata.data)}</span>
              {metadata.data.plugin_name ? (
                <span className="card-chip muted-chip">{metadata.data.plugin_name}</span>
              ) : null}
            </div>
            <dl className="metadata-list">
              {metadata.data.long_name ? (
                <>
                  <dt>Long name</dt>
                  <dd>{metadata.data.long_name}</dd>
                </>
              ) : null}
              {metadata.data.klass ? (
                <>
                  <dt>Klass</dt>
                  <dd>{metadata.data.klass}</dd>
                </>
              ) : null}
              {metadata.data.description ? (
                <>
                  <dt>Description</dt>
                  <dd>{metadata.data.description}</dd>
                </>
              ) : null}
            </dl>

            {metadata.data.pad_templates.length ? (
              <div className="metadata-subsection">
                <span className="field-label">Pad templates</span>
                <ul className="metadata-pill-list">
                  {metadata.data.pad_templates.slice(0, 6).map((pad) => (
                    <li key={`${pad.direction}-${pad.name}`}>
                      {pad.direction} · {pad.name}
                      {pad.presence ? ` · ${pad.presence}` : ''}
                    </li>
                  ))}
                </ul>
              </div>
            ) : null}

            {metadata.data.properties.length ? (
              <div className="metadata-subsection">
                <span className="field-label">
                  GStreamer properties {metadataProperties.length}
                </span>
                <ul className="metadata-property-list">
                  {metadataProperties.map((property) => (
                    <li key={property.name}>
                      <div className="metadata-property-list__header">
                        <span>{property.name}</span>
                        {metadataPropertySummary(property).map((summary) => (
                          <code key={summary}>{summary}</code>
                        ))}
                      </div>
                      {property.description ? <small>{property.description}</small> : null}
                    </li>
                  ))}
                </ul>
              </div>
            ) : null}
          </div>
        )}
      </section>

      <section className="inspector-section">
        <h3>연결</h3>
        <div className="inspector-connection-grid">
          <div>
            <span className="field-label">상류</span>
            <ul className="inspector-connection-list">
              {connections.incoming.length ? (
                connections.incoming.map((connection) => (
                  <li key={connection.id}>{connection.instanceName || connection.factoryName}</li>
                ))
              ) : (
                <li className="muted-copy">상류 노드 없음</li>
              )}
            </ul>
          </div>
          <div>
            <span className="field-label">하류</span>
            <ul className="inspector-connection-list">
              {connections.outgoing.length ? (
                connections.outgoing.map((connection) => (
                  <li key={connection.id}>{connection.instanceName || connection.factoryName}</li>
                ))
              ) : (
                <li className="muted-copy">하류 노드 없음</li>
              )}
            </ul>
          </div>
        </div>
      </section>

      <section className="inspector-section">
        <h3>진단</h3>
        {selectedNode.warnings.length ? (
          <ul className="inspector-diagnostic-list">
            {selectedNode.warnings.map((warning, index) =>
              renderDiagnostic({
                id: `${selectedNode.id}-${index}`,
                severity: 'warning',
                message: warning,
              }),
            )}
          </ul>
        ) : (
          <p className="muted-copy">이 선택에는 노드 수준 경고가 없습니다.</p>
        )}
      </section>
    </aside>
  )
}

export { InspectorPanel }
