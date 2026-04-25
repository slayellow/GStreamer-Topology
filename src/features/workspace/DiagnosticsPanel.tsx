import type { PipelineDiagnostic } from '../../graph/types.ts'

type DiagnosticsPanelProps = {
  diagnostics: PipelineDiagnostic[]
  isOpen: boolean
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
  onToggle,
}: DiagnosticsPanelProps) {
  const itemCountLabel = `${diagnostics.length}개 항목`

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
          <h2>{diagnostics.length ? '파서 피드백' : '파서 진단 없음'}</h2>
          <p className="muted-copy">
            {diagnostics.length
              ? '현재 가져오기에서 복구 가능한 파서 메모를 확인하세요.'
              : '이번 가져오기는 추가 파서 메모 없이 파싱되었습니다.'}
          </p>
        </div>
        <div className="diagnostics-panel__meta">
          <span className="card-chip">{itemCountLabel}</span>
          <span className="diagnostics-panel__state">{isOpen ? '숨기기' : '보기'}</span>
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
                </article>
              ))}
            </div>
          ) : (
            <div className="diagnostic-list diagnostic-list--empty">
              <article className="diagnostic-card severity-info">
                <div className="diagnostic-card__header">
                  <span className="card-chip">정보</span>
                </div>
                <p>파서는 이번 가져오기에 대해 복구 가능한 문제를 보고하지 않았습니다.</p>
              </article>
            </div>
          )}
        </div>
      ) : null}
    </section>
  )
}

export { DiagnosticsPanel }
