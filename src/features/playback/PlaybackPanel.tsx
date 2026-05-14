import { useState } from 'react'
import type {
  PlaybackFrameResponse,
  PlaybackPrepareResponse,
  PlaybackStatusResponse,
  PlaybackStream,
} from '../../app/backend.ts'
import { IconButton } from '../../components/IconButton.tsx'

type PlaybackPanelProps = {
  disabledReason?: string
  documentTitle: string
  frames: Record<string, PlaybackFrameResponse>
  isPreparing: boolean
  isStarting: boolean
  isStopping: boolean
  message?: string
  prepareResult?: PlaybackPrepareResponse
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

function directionLabel(stream: PlaybackStream) {
  return stream.direction === 'sender' ? 'Sender' : 'Receiver'
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

function PlaybackPanel({
  disabledReason,
  documentTitle,
  frames,
  isPreparing,
  isStarting,
  isStopping,
  message,
  prepareResult,
  status,
  onClose,
  onPlay,
  onPrepare,
  onStop,
}: PlaybackPanelProps) {
  const [expandedStreamId, setExpandedStreamId] = useState<string | null>(null)
  const isPlaying = status.state === 'playing'
  const canPrepare = !disabledReason && !isPreparing && !isStarting && !isStopping
  const canPlay = Boolean(prepareResult?.playable) && !isPlaying && !isStarting && !disabledReason
  const canStop = isPlaying && !isStopping
  const expandedFrame = expandedStreamId ? frames[expandedStreamId] : undefined
  const expandedStream = expandedStreamId
    ? prepareResult?.streams.find((stream) => stream.id === expandedStreamId)
    : undefined
  const statusToneName = statusTone(status, prepareResult)
  const helperMessage =
    disabledReason ??
    message ??
    prepareResult?.diagnostic ??
    status.message ??
    'RTP 스트림을 분석하려면 재생 준비를 실행하세요.'
  const shouldShowMessage =
    Boolean(disabledReason) ||
    Boolean(prepareResult?.diagnostic) ||
    status.state === 'error' ||
    status.state === 'stopped' ||
    !prepareResult

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

        {shouldShowMessage ? <p className="playback-window__message">{helperMessage}</p> : null}

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
                  {frames[stream.id]?.data_url ? (
                    <button
                      className="playback-stream-card__frame-button"
                      onClick={() => setExpandedStreamId(stream.id)}
                      type="button"
                    >
                      <img
                        alt={`Stream ${index + 1} preview`}
                        className="playback-stream-card__frame"
                        src={frames[stream.id].data_url ?? undefined}
                      />
                      <span>크게 보기</span>
                    </button>
                  ) : (
                    <div className="playback-stream-card__placeholder">
                      <span>Stream {index + 1}</span>
                      <strong>{mediaKindLabel(stream)}</strong>
                      {isPlaying ? (
                        <small>{frames[stream.id]?.diagnostic ?? 'preview frame 수신 대기 중'}</small>
                      ) : null}
                    </div>
                  )}
                </div>
              </article>
            ))
          ) : (
            <div className="playback-window__empty">
              <span className="card-chip muted-chip">Preview</span>
              <p>Pipeline 재생 준비를 실행하면 감지된 스트림 수만큼 영역을 나눕니다.</p>
            </div>
          )}
        </section>
      </div>
      {expandedFrame?.data_url && expandedStream ? (
        <div className="playback-preview-modal" role="dialog" aria-modal="true">
          <section className="playback-preview-modal__surface panel">
            <header>
              <div>
                <span className="eyebrow">RTP Preview</span>
                <h3>{`${expandedStream.id} · ${directionLabel(expandedStream)}`}</h3>
              </div>
              <IconButton
                icon="close"
                label="확대 Preview 닫기"
                onClick={() => setExpandedStreamId(null)}
              />
            </header>
            <img alt={`${expandedStream.id} expanded preview`} src={expandedFrame.data_url} />
          </section>
        </div>
      ) : null}
    </div>
  )
}

export { PlaybackPanel }
