import type { RuntimeEndpointStatus } from '../app/status.ts'

type ConnectionBadgeProps = {
  label: string
  status: RuntimeEndpointStatus
}

function statusText(status: RuntimeEndpointStatus) {
  switch (status.state) {
    case 'checking':
      return '확인 중'
    case 'connected':
      return '연결됨'
    case 'failed':
      return '연결 실패'
    case 'idle':
      return '미연결'
    case 'unknown':
      return '알 수 없음'
  }
}

function ConnectionBadge({ label, status }: ConnectionBadgeProps) {
  const endpoint = status.host
    ? `${status.host}${status.port ? `:${status.port}` : ''}`
    : undefined
  const details = endpoint ?? status.version ?? status.message ?? statusText(status)

  return (
    <span
      className={`connection-badge connection-badge--${status.state}`}
      title={[label, endpoint, status.version, status.message].filter(Boolean).join(' · ')}
    >
      <span className="status-dot" aria-hidden />
      <span className="connection-badge__copy">
        <strong>{label}</strong>
        <em>{details}</em>
      </span>
    </span>
  )
}

export { ConnectionBadge }
