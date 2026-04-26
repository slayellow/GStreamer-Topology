import { useState, type ChangeEvent } from 'react'
import type { RemoteTargetInput } from '../../app/backend.ts'
import type { GStreamerRuntimeStatus } from '../../app/status.ts'
import { ConnectionBadge } from '../../components/ConnectionBadge.tsx'
import { Icon } from '../../components/Icon.tsx'
import { IconButton } from '../../components/IconButton.tsx'

type ConnectionMode = 'local' | 'remote'

type HomeScreenProps = {
  connectionMode: ConnectionMode
  gstreamerStatus: GStreamerRuntimeStatus
  pipelineText: string
  remoteStatusMessage: string
  remoteTarget: RemoteTargetInput
  statusMessage: string
  onConnectionModeChange: (value: ConnectionMode) => void
  onFileSelected: (event: ChangeEvent<HTMLInputElement>) => void
  onParseText: () => void
  onPipelineTextChange: (value: string) => void
  onProbeRemote: () => void
  onRemoteTargetChange: (value: RemoteTargetInput) => void
}

function HomeScreen({
  connectionMode,
  gstreamerStatus,
  onConnectionModeChange,
  onFileSelected,
  pipelineText,
  remoteStatusMessage,
  remoteTarget,
  statusMessage,
  onParseText,
  onPipelineTextChange,
  onProbeRemote,
  onRemoteTargetChange,
}: HomeScreenProps) {
  const [isRemoteModalOpen, setIsRemoteModalOpen] = useState(false)
  const canUseRemote =
    remoteTarget.host.trim() &&
    remoteTarget.username.trim() &&
    remoteTarget.password.trim()
  const apiStatus =
    connectionMode === 'remote' ? gstreamerStatus.remote : gstreamerStatus.local

  function selectLocalMode() {
    onConnectionModeChange('local')
    setIsRemoteModalOpen(false)
  }

  function openRemoteModal() {
    onConnectionModeChange('remote')
    setIsRemoteModalOpen(true)
  }

  return (
    <main className="app-shell app-shell--home">
      <section className="home-screen home-screen--focused">
        <section className="launcher-card launcher-card--single panel">
          <header className="launcher-card__topbar">
            <h1>GStreamer Topology</h1>

            <div className="home-toolbar" aria-label="시작 화면 작업">
              <ConnectionBadge label="GStreamer API" status={apiStatus} />

              <label
                aria-label="로컬 파이프라인 파일 열기"
                className="icon-button icon-upload-button"
                htmlFor="local-pipeline-file"
                title="로컬 파이프라인 파일 열기"
              >
                <Icon name="folderOpen" />
                <span className="sr-only">로컬 파이프라인 파일 열기</span>
                <input
                  accept=".pld,.txt"
                  id="local-pipeline-file"
                  onChange={onFileSelected}
                  type="file"
                />
              </label>

              <IconButton
                active={connectionMode === 'local'}
                icon="monitor"
                label="Local 모드"
                onClick={selectLocalMode}
              />
              <IconButton
                active={connectionMode === 'remote'}
                badge={gstreamerStatus.remote.state === 'connected' ? '•' : undefined}
                icon="server"
                label="Remote Server 연결"
                onClick={openRemoteModal}
              />
            </div>
          </header>

          <section className="launcher-card__main">
            <textarea
              className="text-area pipeline-input"
              onChange={(event) => onPipelineTextChange(event.target.value)}
              placeholder="videotestsrc pattern=smpte ! videoconvert ! autovideosink"
              value={pipelineText}
            />

            <button
              className="primary-button topology-generate-button"
              disabled={!pipelineText.trim()}
              onClick={onParseText}
              type="button"
            >
              토폴로지 생성
            </button>

            {statusMessage ? (
              <p className="home-status-line">{statusMessage}</p>
            ) : null}
          </section>
        </section>
      </section>

      {isRemoteModalOpen ? (
        <div className="remote-modal" role="dialog" aria-modal="true" aria-labelledby="remote-modal-title">
          <section className="remote-modal__card remote-modal__card--compact panel">
            <header className="remote-modal__header">
              <h2 id="remote-modal-title">Remote Server 연결</h2>
              <IconButton
                icon="close"
                label="Remote Server 창 닫기"
                onClick={() => setIsRemoteModalOpen(false)}
              />
            </header>

            <section className="remote-card remote-card--compact">
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

              <div className="remote-card__footer">
                <button
                  className="primary-button remote-connect-button"
                  disabled={!canUseRemote}
                  onClick={onProbeRemote}
                  type="button"
                >
                  접속
                </button>
                <ConnectionBadge label="Remote Server" status={gstreamerStatus.remote} />
              </div>

              {remoteStatusMessage ? (
                <p className="remote-status-line">{remoteStatusMessage}</p>
              ) : null}
            </section>
          </section>
        </div>
      ) : null}
    </main>
  )
}

export { HomeScreen }
