export type RuntimeStatusState = 'checking' | 'connected' | 'failed' | 'idle' | 'unknown'

export type RuntimeEndpointStatus = {
  detail?: string
  host?: string
  message?: string
  port?: number
  state: RuntimeStatusState
  version?: string
}

export type GStreamerRuntimeStatus = {
  local: RuntimeEndpointStatus
  remote: RuntimeEndpointStatus
}

export function firstVersionLine(versionOutput?: string | null) {
  return versionOutput?.split('\n').find((line) => line.trim())?.trim()
}
