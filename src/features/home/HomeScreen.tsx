import type { ChangeEvent } from 'react'

type HomeScreenProps = {
  isTauri: boolean
  pipelineText: string
  statusMessage: string
  onFileSelected: (event: ChangeEvent<HTMLInputElement>) => void
  onParseText: () => void
  onPipelineTextChange: (value: string) => void
}

function HomeScreen({
  isTauri,
  onFileSelected,
  pipelineText,
  statusMessage,
  onParseText,
  onPipelineTextChange,
}: HomeScreenProps) {
  return (
    <main className="app-shell">
      <section className="home-screen home-screen--focused">
        <section className="launcher-card panel">
          <div className="launcher-card__header">
            <div className="launcher-card__badges">
              <span className="card-chip muted-chip">
                {isTauri ? '데스크톱 백엔드 준비됨' : '브라우저 미리보기 전용'}
              </span>
            </div>

            <div className="launcher-card__copy">
              <div className="eyebrow">로컬 파이프라인</div>
              <h1>GStreamer Topology</h1>
              <p className="hero-body">
                로컬 `.pld`, `.txt`, `.rtf` 파일을 가져오거나 파이프라인 텍스트를
                붙여넣어 캔버스 중심 토폴로지 보기로 변환하세요. 예제 파일은 저장소의{' '}
                <code>fixtures/pipelines/</code> 폴더에 있습니다.
              </p>
            </div>
          </div>

          <div className="launcher-card__body">
            <div className="launcher-card__actions">
              <label className="primary-button upload-action" htmlFor="local-pipeline-file">
                로컬 파이프라인 열기
                <input
                  id="local-pipeline-file"
                  onChange={onFileSelected}
                  type="file"
                  accept=".txt,.pld,.rtf,.pld.rtf"
                />
              </label>

              <div className="launcher-card__formats">
                <span className="card-chip muted-chip">.pld</span>
                <span className="card-chip muted-chip">.txt</span>
                <span className="card-chip muted-chip">.rtf</span>
              </div>
            </div>

            <div className="launcher-divider" role="presentation">
              <span>또는 파이프라인 텍스트 붙여넣기</span>
            </div>

            <div className="launcher-card__paste">
              <textarea
                className="text-area"
                onChange={(event) => onPipelineTextChange(event.target.value)}
                placeholder="videotestsrc pattern=smpte ! videoconvert ! autovideosink"
                value={pipelineText}
              />

              <div className="launcher-card__footer">
                <button
                  className="secondary-button"
                  disabled={!pipelineText.trim()}
                  onClick={onParseText}
                  type="button"
                >
                  토폴로지 생성
                </button>

                <div className="status-strip">
                  <span className="card-chip muted-chip">
                    {isTauri ? '로컬 파싱 경로' : '데스크톱 런타임 필요'}
                  </span>
                  <span className="status-message">{statusMessage}</span>
                </div>
              </div>
            </div>
          </div>
        </section>
      </section>
    </main>
  )
}

export { HomeScreen }
