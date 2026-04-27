import { useEffect, useState, type ChangeEvent } from 'react'
import {
  isTauriRuntime,
  parsePipelineText,
  probeLocalGStreamer,
  probeRemoteTarget,
  type RemoteTargetInput,
} from './backend.ts'
import {
  firstVersionLine,
  type GStreamerRuntimeStatus,
} from './status.ts'
import { HomeScreen } from '../features/home/HomeScreen.tsx'
import { WorkspaceShell } from '../features/workspace/WorkspaceShell.tsx'
import { toViewModel } from '../graph/fromBackend.ts'
import type { PipelineDocumentViewModel } from '../graph/types.ts'

type ConnectionMode = 'local' | 'remote'

function localGStreamerFailureStatus(diagnostic?: string | null) {
  return {
    detail: diagnostic ?? undefined,
    message: diagnostic?.includes('Checked ')
      ? 'Local GStreamer API 경로 미탐지'
      : diagnostic ?? '로컬 gst-inspect-1.0을 찾지 못했습니다.',
    state: 'failed' as const,
  }
}

function AppShell() {
  const [activeDocument, setActiveDocument] = useState<PipelineDocumentViewModel | null>(null)
  const [connectionMode, setConnectionMode] = useState<ConnectionMode>('local')
  const [pipelineText, setPipelineText] = useState('')
  const [pipelineSourceName, setPipelineSourceName] = useState('붙여넣은 파이프라인')
  const [remoteTarget, setRemoteTarget] = useState<RemoteTargetInput>({
    host: '',
    port: 22,
    username: '',
    password: '',
  })
  const [activeRemoteTarget, setActiveRemoteTarget] = useState<RemoteTargetInput | null>(null)
  const [verifiedRemoteTarget, setVerifiedRemoteTarget] = useState<RemoteTargetInput | null>(null)
  const [gstreamerStatus, setGstreamerStatus] = useState<GStreamerRuntimeStatus>(() => ({
    local: {
      message: isTauriRuntime()
        ? '로컬 gst-inspect 확인 중...'
        : '데스크톱 Tauri 런타임에서 확인할 수 있습니다.',
      state: isTauriRuntime() ? 'checking' : 'unknown',
    },
    remote: {
      message: '원격 미연결',
      state: 'idle',
    },
  }))
  const [remoteStatusMessage, setRemoteStatusMessage] = useState('')
  const [statusMessage, setStatusMessage] = useState('')

  useEffect(() => {
    if (!isTauriRuntime()) {
      return
    }

    let isCancelled = false
    probeLocalGStreamer()
      .then((response) => {
        if (isCancelled) {
          return
        }

        const version = firstVersionLine(response.version_output)
        setGstreamerStatus((current) => ({
          ...current,
          local: response.available
            ? {
                message: '로컬 GStreamer API 연결됨',
                state: 'connected',
                version: version ?? 'GStreamer version 확인됨',
              }
            : {
                ...localGStreamerFailureStatus(response.diagnostic),
              },
        }))
      })
      .catch((error) => {
        if (isCancelled) {
          return
        }

        console.error(error)
        setGstreamerStatus((current) => ({
          ...current,
          local: {
            message: '로컬 GStreamer API 확인에 실패했습니다.',
            state: 'failed',
          },
        }))
      })

    return () => {
      isCancelled = true
    }
  }, [])

  async function handlePipelineImport(rawText: string, sourceName: string) {
    if (!isTauriRuntime()) {
      setStatusMessage('로컬 파서 흐름은 데스크톱 Tauri 런타임이 필요합니다.')
      return
    }

    setStatusMessage(`${sourceName} 파싱 중...`)

    try {
      const backendDocument = await parsePipelineText(rawText, sourceName)
      const viewModel = await toViewModel(backendDocument)
      setActiveDocument(viewModel)
      setActiveRemoteTarget(
        connectionMode === 'remote' && verifiedRemoteTarget
          ? verifiedRemoteTarget
          : null,
      )
      setStatusMessage(
        viewModel.diagnostics.length
          ? `${sourceName}에서 토폴로지를 생성했습니다. 확인할 파서 진단 ${viewModel.diagnostics.length}건이 있습니다.`
          : `${sourceName}에서 노드 ${viewModel.graph.nodes.length}개와 엣지 ${viewModel.graph.edges.length}개를 파싱했습니다.`,
      )
    } catch (error) {
      console.error(error)
      setStatusMessage(
        '불러오기에 실패했습니다. 파일 내용이나 파이프라인 구문을 확인한 뒤 다시 시도해 주세요.',
      )
    }
  }

  async function handleFileSelected(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0]
    if (!file) {
      return
    }

    const rawText = await file.text()
    setPipelineSourceName(file.name)

    if (!isTauriRuntime()) {
      setPipelineText(rawText)
      setStatusMessage(`${file.name} 내용을 입력 영역에 불러왔습니다.`)
      event.target.value = ''
      return
    }

    setStatusMessage(`${file.name} 내용을 정규화하는 중...`)

    try {
      const backendDocument = await parsePipelineText(rawText, file.name)
      setPipelineText(backendDocument.normalized_text)
      setStatusMessage(
        backendDocument.diagnostics.length
          ? `${file.name} 내용을 입력 영역에 불러왔습니다. 확인할 파서 진단 ${backendDocument.diagnostics.length}건이 있습니다.`
          : `${file.name} 내용을 입력 영역에 불러왔습니다. 필요하면 수정한 뒤 토폴로지를 생성하세요.`,
      )
    } catch (error) {
      console.error(error)
      setPipelineText(rawText)
      setStatusMessage(
        `${file.name} 정규화에 실패해 원문을 입력 영역에 불러왔습니다. 내용을 확인해 주세요.`,
      )
    }

    event.target.value = ''
  }

  async function handleParsePastedText() {
    if (!pipelineText.trim()) {
      setStatusMessage('파싱하기 전에 파이프라인 텍스트를 붙여넣으세요.')
      return
    }

    await handlePipelineImport(pipelineText, pipelineSourceName)
  }

  function sanitizeRemoteTarget(): RemoteTargetInput {
    return {
      ...remoteTarget,
      host: remoteTarget.host.trim(),
      username: remoteTarget.username.trim(),
      port: Number(remoteTarget.port || 22),
    }
  }

  function handlePipelineTextChange(value: string) {
    setPipelineText(value)
    setPipelineSourceName('붙여넣은 파이프라인')
  }

  function handleRemoteTargetChange(value: RemoteTargetInput) {
    setRemoteTarget(value)
    setVerifiedRemoteTarget(null)
    setGstreamerStatus((current) => ({
      ...current,
      remote: {
        host: value.host.trim() || undefined,
        message: '원격 미연결',
        port: Number(value.port || 22),
        state: 'idle',
      },
    }))
    setRemoteStatusMessage('')
  }

  async function handleProbeRemote() {
    if (!isTauriRuntime()) {
      setRemoteStatusMessage('원격 접속은 데스크톱 Tauri 런타임에서만 사용할 수 있습니다.')
      return
    }

    const request = sanitizeRemoteTarget()
    if (!request.host || !request.username || !request.password) {
      setRemoteStatusMessage('원격 IP, 계정 ID, PW를 모두 입력해 주세요.')
      return
    }

    setRemoteStatusMessage(`${request.host} 접속 및 gst-inspect 확인 중...`)
    setGstreamerStatus((current) => ({
      ...current,
      remote: {
        host: request.host,
        message: 'Remote Server 연결 확인 중...',
        port: request.port,
        state: 'checking',
      },
    }))
    try {
      const response = await probeRemoteTarget(request, 'videotestsrc')
      const version = firstVersionLine(response.version_output)
      setGstreamerStatus((current) => ({
        ...current,
        remote: {
          host: response.host,
          message: 'Remote Server 연결됨',
          port: response.port,
          state: 'connected',
          version: version ?? 'GStreamer version 확인됨',
        },
      }))
      setVerifiedRemoteTarget({
        ...request,
        host: response.host,
        port: response.port,
      })
      setRemoteStatusMessage(
        `${response.host} 접속 성공. ${response.version_output.split('\n')[0] ?? 'GStreamer 정보를 확인했습니다.'}`,
      )
    } catch (error) {
      console.error(error)
      setGstreamerStatus((current) => ({
        ...current,
        remote: {
          host: request.host,
          message: 'Remote Server 연결 실패',
          port: request.port,
          state: 'failed',
        },
      }))
      setVerifiedRemoteTarget(null)
      setRemoteStatusMessage('원격 접속 또는 gst-inspect 확인에 실패했습니다. IP, 계정, PW, 네트워크를 확인해 주세요.')
    }
  }

  if (!activeDocument) {
    return (
      <HomeScreen
        connectionMode={connectionMode}
        gstreamerStatus={gstreamerStatus}
        pipelineText={pipelineText}
        statusMessage={statusMessage}
        remoteStatusMessage={remoteStatusMessage}
        remoteTarget={remoteTarget}
        onConnectionModeChange={setConnectionMode}
        onFileSelected={handleFileSelected}
        onParseText={handleParsePastedText}
        onPipelineTextChange={handlePipelineTextChange}
        onProbeRemote={handleProbeRemote}
        onRemoteTargetChange={handleRemoteTargetChange}
      />
    )
  }

  return (
    <WorkspaceShell
      key={activeDocument.id}
      document={activeDocument}
      gstreamerStatus={gstreamerStatus}
      remoteTarget={activeRemoteTarget}
      onBackHome={() => setActiveDocument(null)}
    />
  )
}

export default AppShell
