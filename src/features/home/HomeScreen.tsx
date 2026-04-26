import type { ChangeEvent } from 'react'
import type { RemoteTargetInput } from '../../app/backend.ts'

type HomeScreenProps = {
  isTauri: boolean
  pipelineText: string
  remotePath: string
  remoteStatusMessage: string
  remoteTarget: RemoteTargetInput
  statusMessage: string
  onFileSelected: (event: ChangeEvent<HTMLInputElement>) => void
  onLoadRemotePipeline: () => void
  onParseText: () => void
  onPipelineTextChange: (value: string) => void
  onProbeRemote: () => void
  onRemotePathChange: (value: string) => void
  onRemoteTargetChange: (value: RemoteTargetInput) => void
}

function HomeScreen({
  isTauri,
  onFileSelected,
  onLoadRemotePipeline,
  pipelineText,
  remotePath,
  remoteStatusMessage,
  remoteTarget,
  statusMessage,
  onParseText,
  onPipelineTextChange,
  onProbeRemote,
  onRemotePathChange,
  onRemoteTargetChange,
}: HomeScreenProps) {
  const canUseRemote =
    remoteTarget.host.trim() &&
    remoteTarget.username.trim() &&
    remoteTarget.password.trim()

  return (
    <main className="app-shell app-shell--home">
      <section className="home-screen home-screen--focused">
        <section className="launcher-card panel">
          <div className="launcher-card__header">
            <div className="launcher-card__badges">
              <span className="card-chip muted-chip">
                {isTauri ? '데스크톱 백엔드 준비됨' : '브라우저 미리보기 전용'}
              </span>
            </div>

            <div className="launcher-card__copy">
              <div className="eyebrow">로컬 파일 / Remote Server</div>
              <h1>GStreamer Topology</h1>
              <p className="hero-body">
                로컬 `.pld`, `.txt`, `.rtf` 파일을 가져오거나 파이프라인 텍스트를
                붙여넣고, Remote Server의 OE-Linux 파이프라인도 같은 캔버스에서
                확인하세요. 예제 파일은 <code>fixtures/pipelines/</code> 폴더에 있습니다.
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

            <div className="launcher-divider" role="presentation">
              <span>또는 Remote Server에서 불러오기</span>
            </div>

            <section className="remote-card">
              <div className="remote-card__grid">
                <label>
                  <span className="field-label">IP</span>
                  <input
                    className="text-field"
                    onChange={(event) =>
                      onRemoteTargetChange({
                        ...remoteTarget,
                        host: event.target.value,
                      })
                    }
                    placeholder="192.168.0.10"
                    value={remoteTarget.host}
                  />
                </label>
                <label>
                  <span className="field-label">Port</span>
                  <input
                    className="text-field"
                    min={1}
                    onChange={(event) =>
                      onRemoteTargetChange({
                        ...remoteTarget,
                        port: Number(event.target.value || 22),
                      })
                    }
                    type="number"
                    value={remoteTarget.port}
                  />
                </label>
                <label>
                  <span className="field-label">ID</span>
                  <input
                    className="text-field"
                    onChange={(event) =>
                      onRemoteTargetChange({
                        ...remoteTarget,
                        username: event.target.value,
                      })
                    }
                    placeholder="root"
                    value={remoteTarget.username}
                  />
                </label>
                <label>
                  <span className="field-label">PW</span>
                  <input
                    className="text-field"
                    onChange={(event) =>
                      onRemoteTargetChange({
                        ...remoteTarget,
                        password: event.target.value,
                      })
                    }
                    type="password"
                    value={remoteTarget.password}
                  />
                </label>
              </div>

              <label className="remote-card__path">
                <span className="field-label">Remote Server Pipeline 파일 경로</span>
                <input
                  className="text-field"
                  onChange={(event) => onRemotePathChange(event.target.value)}
                  placeholder="/home/root/pipeline.pld"
                  value={remotePath}
                />
              </label>

              <div className="launcher-card__footer">
                <div className="remote-card__actions">
                  <button
                    className="secondary-button"
                    disabled={!canUseRemote}
                    onClick={onProbeRemote}
                    type="button"
                  >
                    원격 접속 확인
                  </button>
                  <button
                    className="secondary-button"
                    disabled={!canUseRemote || !remotePath.trim()}
                    onClick={onLoadRemotePipeline}
                    type="button"
                  >
                    원격 토폴로지 생성
                  </button>
                </div>

                <div className="status-strip">
                  <span className="card-chip muted-chip">읽기 전용 SSH/SFTP</span>
                  <span className="status-message">{remoteStatusMessage}</span>
                </div>
              </div>
            </section>
          </div>
        </section>
      </section>
    </main>
  )
}

export { HomeScreen }
