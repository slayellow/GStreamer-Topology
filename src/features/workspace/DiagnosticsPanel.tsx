import type { PipelineDiagnostic } from '../../graph/types.ts'

type DiagnosticsPanelProps = {
  diagnostics: PipelineDiagnostic[]
  isOpen: boolean
  onShowSource?: (diagnostic: PipelineDiagnostic) => void
  onToggle: () => void
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

function DiagnosticsPanel({
  diagnostics,
  isOpen,
  onShowSource,
  onToggle,
}: DiagnosticsPanelProps) {
  const itemCountLabel = `${diagnostics.length} notes`

  return (
    <section className={`workspace-panel diagnostics-panel${isOpen ? ' is-open' : ''}`}>
      <button
        aria-expanded={isOpen}
        className="diagnostics-panel__toggle"
        onClick={onToggle}
        type="button"
      >
        <div>
          <div className="eyebrow">진단</div>
          <h2>{diagnostics.length ? 'Parser Notes' : 'No Parser Notes'}</h2>
          <p className="muted-copy">
            {diagnostics.length
              ? '파이프라인 텍스트를 토폴로지로 해석하며 복구한 부분을 보여줍니다. 실제 GStreamer 실행 검증은 아닙니다.'
              : '텍스트 파서는 복구 가능한 문제를 보고하지 않았습니다. 실제 실행 가능 여부는 대상 GStreamer 환경에서 별도 확인이 필요합니다.'}
          </p>
        </div>
        <div className="diagnostics-panel__meta">
          <span className="card-chip">{itemCountLabel}</span>
          <span className="diagnostics-panel__state">{isOpen ? 'Hide' : 'Show'}</span>
        </div>
      </button>

      {isOpen ? (
        <div className="diagnostics-panel__content">
          {diagnostics.length ? (
            <div className="diagnostic-list">
              {diagnostics.map((diagnostic) => (
                <article
                  className={`diagnostic-card severity-${diagnostic.severity}`}
                  key={diagnostic.id}
                >
                  <div className="diagnostic-card__header">
                    <span className="card-chip">{diagnosticSeverityLabel(diagnostic.severity)}</span>
                    {diagnostic.nodeId ? <span>{diagnostic.nodeId}</span> : null}
                  </div>
                  <p>{diagnostic.message}</p>
                  <p className="diagnostic-card__action-copy">
                    {diagnostic.sourceSpan
                      ? '원문 위치를 함께 확인해 parser가 어느 구간을 복구했는지 비교하세요.'
                      : '원문 위치가 없는 진단입니다. 전체 pipeline 문맥에서 연결 또는 토큰을 확인하세요.'}
                  </p>
                  {diagnostic.sourceSpan && onShowSource ? (
                    <button
                      className="diagnostic-card__source-button"
                      onClick={() => onShowSource(diagnostic)}
                      type="button"
                    >
                      원문 {diagnostic.sourceSpan.lineStart}-{diagnostic.sourceSpan.lineEnd}행 보기
                    </button>
                  ) : null}
                </article>
              ))}
            </div>
          ) : (
            <div className="diagnostic-list diagnostic-list--empty">
              <article className="diagnostic-card severity-info">
                <div className="diagnostic-card__header">
                  <span className="card-chip">정보</span>
                </div>
                <p>
                  파서는 이번 가져오기에 대해 복구 가능한 문제를 보고하지 않았습니다.
                  실제 실행 가능 여부는 대상 GStreamer 환경에서 별도 확인이 필요합니다.
                </p>
              </article>
            </div>
          )}
        </div>
      ) : null}
    </section>
  )
}

export { DiagnosticsPanel }
