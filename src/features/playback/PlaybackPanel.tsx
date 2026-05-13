import type {
  PlaybackPrepareResponse,
  PlaybackStatusResponse,
  PlaybackStream,
} from '../../app/backend.ts'
import { IconButton } from '../../components/IconButton.tsx'

type PlaybackPanelProps = {
  disabledReason?: string
  documentTitle: string
  isPreparing: boolean
  isStarting: boolean
  isStopping: boolean
  message?: string
  prepareResult?: PlaybackPrepareResponse
  rawText: string
  status: PlaybackStatusResponse
  onClose: () => void
  onPlay: () => void
  onPrepare: () => void
  onStop: () => void
}

function statusLabel(status: PlaybackStatusResponse, prepareResult?: PlaybackPrepareResponse) {
  if (status.state === 'playing') {
    return '재생 중'
  }

  if (status.state === 'error') {
    return '오류'
  }

  if (prepareResult?.playable) {
    return '재생 가능'
  }

  if (prepareResult && !prepareResult.playable) {
    return '재생 불가'
  }

  if (status.state === 'stopped') {
    return '정지됨'
  }

  return '준비 전'
}

function statusTone(status: PlaybackStatusResponse, prepareResult?: PlaybackPrepareResponse) {
  if (status.state === 'playing') {
    return 'success'
  }

  if (status.state === 'error' || (prepareResult && !prepareResult.playable)) {
    return 'error'
  }

  if (prepareResult?.playable) {
    return 'info'
  }

  return 'muted'
}

function protocolLabel(stream: PlaybackStream) {
  return stream.protocol.toUpperCase()
}

function mediaKindLabel(stream: PlaybackStream) {
  switch (stream.media_kind) {
    case 'audio':
      return 'Audio'
    case 'video':
      return 'Video'
    case 'unknown':
      return 'Unknown'
  }
}

function endpointLabel(stream: PlaybackStream) {
  if (stream.uri) {
    return stream.uri
  }

  return `${stream.host ?? '0.0.0.0'}:${stream.port ?? '-'}`
}

function PlaybackPanel({
  disabledReason,
  documentTitle,
  isPreparing,
  isStarting,
  isStopping,
  message,
  prepareResult,
  rawText,
  status,
  onClose,
  onPlay,
  onPrepare,
  onStop,
}: PlaybackPanelProps) {
  const isPlaying = status.state === 'playing'
  const canPrepare = !disabledReason && !isPreparing && !isStarting && !isStopping
  const canPlay = Boolean(prepareResult?.playable) && !isPlaying && !isStarting && !disabledReason
  const canStop = isPlaying && !isStopping
  const generatedPipeline = prepareResult?.generated_pipeline ?? status.command
  const statusToneName = statusTone(status, prepareResult)
  const helperMessage =
    disabledReason ??
    message ??
    prepareResult?.diagnostic ??
    status.message ??
    'RTP/RTSP 스트림을 분석하려면 재생 준비를 실행하세요.'

  return (
    <div className="playback-window" role="dialog" aria-modal="true" aria-label="Pipeline Playback">
      <div className="playback-window__surface panel">
        <header className="playback-window__header">
          <div>
            <span className="eyebrow">Pipeline Playback</span>
            <h2>{documentTitle}</h2>
          </div>
          <div className="playback-window__header-actions">
            <span className={`card-chip playback-status playback-status--${statusToneName}`}>
              {statusLabel(status, prepareResult)}
            </span>
            <IconButton icon="close" label="Playback 창 닫기" onClick={onClose} />
          </div>
        </header>

        <div className="playback-window__summary">
          <span className="card-chip">
            Local GStreamer {prepareResult?.available === false ? '미감지' : '대기'}
          </span>
          <span className="card-chip">
            Streams {prepareResult?.streams.length ?? 0}
          </span>
          {status.pid ? <span className="card-chip muted-chip">PID {status.pid}</span> : null}
        </div>

        <p className="playback-window__message">{helperMessage}</p>

        <div className="playback-window__controls">
          <button disabled={!canPrepare} onClick={onPrepare} type="button">
            {isPreparing ? '분석 중...' : 'Pipeline 재생 준비'}
          </button>
          <button disabled={!canPlay} onClick={onPlay} type="button">
            {isStarting ? '재생 시작 중...' : '재생'}
          </button>
          <button disabled={!canStop} onClick={onStop} type="button">
            {isStopping ? '정지 중...' : '중지'}
          </button>
        </div>

        <section className="playback-window__preview" aria-label="Media Preview">
          {prepareResult?.streams.length ? (
            prepareResult.streams.map((stream, index) => (
              <article className="playback-stream-card" key={stream.id}>
                <div className="playback-stream-card__screen">
                  <span>Stream {index + 1}</span>
                  <strong>{mediaKindLabel(stream)}</strong>
                </div>
                <div className="playback-stream-card__meta">
                  <span className="card-chip">{protocolLabel(stream)}</span>
                  <span className="card-chip muted-chip">{endpointLabel(stream)}</span>
                </div>
                {stream.caps ? <code>{stream.caps}</code> : null}
              </article>
            ))
          ) : (
            <div className="playback-window__empty">
              <span className="card-chip muted-chip">Preview</span>
              <p>Pipeline 재생 준비를 실행하면 감지된 스트림 수만큼 영역을 나눕니다.</p>
            </div>
          )}
        </section>

        <section className="playback-window__code-grid">
          <div>
            <span className="field-label">Generated Playback Pipeline</span>
            <pre>{generatedPipeline ?? '아직 생성된 재생 Pipeline이 없습니다.'}</pre>
          </div>
          <div>
            <span className="field-label">PLD Source</span>
            <pre>{rawText}</pre>
          </div>
        </section>
      </div>
    </div>
  )
}

export { PlaybackPanel }
