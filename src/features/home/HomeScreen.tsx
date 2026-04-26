import { useState, type ChangeEvent } from 'react'
import type { RemoteTargetInput } from '../../app/backend.ts'
import type { GStreamerRuntimeStatus } from '../../app/status.ts'
import { ConnectionBadge } from '../../components/ConnectionBadge.tsx'
import { Icon } from '../../components/Icon.tsx'
import { IconButton } from '../../components/IconButton.tsx'

type HomeScreenProps = {
  gstreamerStatus: GStreamerRuntimeStatus
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
  gstreamerStatus,
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
  const [isRemoteModalOpen, setIsRemoteModalOpen] = useState(false)
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
              <ConnectionBadge label="GStreamer API" status={gstreamerStatus.local} />
              <ConnectionBadge label="Remote Server" status={gstreamerStatus.remote} />
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
              <label
                aria-label="로컬 파이프라인 파일 열기"
                className="icon-button icon-upload-button icon-button--primary"
                htmlFor="local-pipeline-file"
                title="로컬 파이프라인 파일 열기"
              >
                <Icon name="folderOpen" />
                <span className="sr-only">로컬 파이프라인 파일 열기</span>
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
              <span>Remote Server 연결</span>
            </div>

            <section className="remote-entry">
              <button
                className="remote-entry__button"
                onClick={() => setIsRemoteModalOpen(true)}
                type="button"
              >
                <span className="remote-entry__icon">
                  <Icon name="server" />
                </span>
                <span>
                  <strong>Remote Server</strong>
                  <em>대상 장비의 GStreamer metadata를 읽기 전용으로 확인합니다.</em>
                </span>
              </button>
              <ConnectionBadge label="Remote Server" status={gstreamerStatus.remote} />
              <span className="status-message">{remoteStatusMessage}</span>
            </section>
          </div>
        </section>
      </section>

      {isRemoteModalOpen ? (
        <div className="remote-modal" role="dialog" aria-modal="true" aria-labelledby="remote-modal-title">
          <section className="remote-modal__card panel">
            <header className="remote-modal__header">
              <div>
                <div className="eyebrow">Remote Server</div>
                <h2 id="remote-modal-title">Remote Server 연결</h2>
                <p className="muted-copy">
                  OE-Linux 대상 장비에서 GStreamer API와 Pipeline 파일을 읽기 전용으로
                  확인합니다.
                </p>
              </div>
              <IconButton
                icon="close"
                label="Remote Server 창 닫기"
                onClick={() => setIsRemoteModalOpen(false)}
              />
            </header>

            <section className="remote-card">
              <div className="remote-card__grid">
                <label>
                  <span className="field-label">IP / Host</span>
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
                <span className="field-label">Remote Pipeline 파일 경로</span>
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
                    원격 파일 미리보기
                  </button>
                </div>

                <div className="status-strip">
                  <span className="card-chip muted-chip">읽기 전용 SSH/SFTP</span>
                  <ConnectionBadge label="Remote Server" status={gstreamerStatus.remote} />
                  <span className="status-message">{remoteStatusMessage}</span>
                </div>
              </div>
            </section>
          </section>
        </div>
      ) : null}
    </main>
  )
}

export { HomeScreen }
