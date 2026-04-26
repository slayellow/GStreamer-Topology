import { useEffect, useState, type ChangeEvent } from 'react'
import {
  isTauriRuntime,
  loadRemotePipeline,
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
import {
  ImportPreviewScreen,
  type ImportPreviewViewModel,
} from '../features/import-preview/ImportPreviewScreen.tsx'
import { WorkspaceShell } from '../features/workspace/WorkspaceShell.tsx'
import { toViewModel } from '../graph/fromBackend.ts'
import type { PipelineDocumentViewModel } from '../graph/types.ts'

function AppShell() {
  const [activeDocument, setActiveDocument] = useState<PipelineDocumentViewModel | null>(null)
  const [pendingPreview, setPendingPreview] = useState<ImportPreviewViewModel | null>(null)
  const [isGeneratingPreview, setIsGeneratingPreview] = useState(false)
  const [pipelineText, setPipelineText] = useState('')
  const [remoteTarget, setRemoteTarget] = useState<RemoteTargetInput>({
    host: '',
    port: 22,
    username: '',
    password: '',
  })
  const [remotePath, setRemotePath] = useState('')
  const [activeRemoteTarget, setActiveRemoteTarget] = useState<RemoteTargetInput | null>(null)
  const [previewRemoteTarget, setPreviewRemoteTarget] = useState<RemoteTargetInput | null>(null)
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
  const [remoteStatusMessage, setRemoteStatusMessage] = useState(
    '원격 OE-Linux 장비가 준비되면 IP, 계정, 경로를 입력해 읽기 전용으로 불러올 수 있습니다.',
  )
  const [statusMessage, setStatusMessage] = useState(
    '로컬 파이프라인 파일을 열거나 텍스트를 붙여넣어 토폴로지로 변환하세요.',
  )

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
                message: response.diagnostic ?? '로컬 gst-inspect-1.0을 찾지 못했습니다.',
                state: 'failed',
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
      setActiveRemoteTarget(null)
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

  async function handlePipelinePreview(
    rawText: string,
    sourceName: string,
    remoteContext: RemoteTargetInput | null = null,
  ) {
    if (!isTauriRuntime()) {
      setStatusMessage('파일 미리보기는 데스크톱 Tauri 런타임이 필요합니다.')
      return
    }

    setStatusMessage(`${sourceName} 미리보기 준비 중...`)

    try {
      const backendDocument = await parsePipelineText(rawText, sourceName)
      const viewModel = await toViewModel(backendDocument)
      setPendingPreview({
        document: viewModel,
        remoteHost: remoteContext?.host,
        sourceName,
        text: viewModel.normalizedText,
      })
      setPreviewRemoteTarget(remoteContext)
      setActiveDocument(null)
      setActiveRemoteTarget(null)
      setStatusMessage(
        viewModel.diagnostics.length
          ? `${sourceName} 미리보기를 준비했습니다. 확인할 파서 진단 ${viewModel.diagnostics.length}건이 있습니다.`
          : `${sourceName} 미리보기를 준비했습니다. 토폴로지 생성 전 원문을 확인하세요.`,
      )
    } catch (error) {
      console.error(error)
      setStatusMessage(
        '미리보기를 준비하지 못했습니다. 파일 내용이나 파이프라인 구문을 확인한 뒤 다시 시도해 주세요.',
      )
    }
  }

  async function handleFileSelected(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0]
    if (!file) {
      return
    }

    const rawText = await file.text()
    await handlePipelinePreview(rawText, file.name)
    event.target.value = ''
  }

  async function handleParsePastedText() {
    if (!pipelineText.trim()) {
      setStatusMessage('파싱하기 전에 파이프라인 텍스트를 붙여넣으세요.')
      return
    }

    await handlePipelineImport(pipelineText, '붙여넣은 파이프라인')
  }

  async function handleGeneratePreviewTopology() {
    if (!pendingPreview) {
      return
    }

    if (!pendingPreview.text.trim()) {
      setStatusMessage('토폴로지를 생성하기 전에 파이프라인 텍스트를 확인해 주세요.')
      return
    }

    setIsGeneratingPreview(true)
    try {
      const backendDocument = await parsePipelineText(
        pendingPreview.text,
        pendingPreview.sourceName,
      )
      const viewModel = await toViewModel(backendDocument)
      setActiveDocument(viewModel)
      setActiveRemoteTarget(previewRemoteTarget)
      setPendingPreview(null)
      setPreviewRemoteTarget(null)
      setStatusMessage(
        viewModel.diagnostics.length
          ? `${pendingPreview.sourceName}에서 토폴로지를 생성했습니다. 확인할 파서 진단 ${viewModel.diagnostics.length}건이 있습니다.`
          : `${pendingPreview.sourceName}에서 노드 ${viewModel.graph.nodes.length}개와 엣지 ${viewModel.graph.edges.length}개를 파싱했습니다.`,
      )
    } catch (error) {
      console.error(error)
      setStatusMessage('토폴로지를 생성하지 못했습니다. 미리보기 텍스트를 확인해 주세요.')
    } finally {
      setIsGeneratingPreview(false)
    }
  }

  function sanitizeRemoteTarget(): RemoteTargetInput {
    return {
      ...remoteTarget,
      host: remoteTarget.host.trim(),
      username: remoteTarget.username.trim(),
      port: Number(remoteTarget.port || 22),
    }
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
      setRemoteStatusMessage('원격 접속 또는 gst-inspect 확인에 실패했습니다. IP, 계정, PW, 네트워크를 확인해 주세요.')
    }
  }

  async function handleLoadRemotePipeline() {
    if (!isTauriRuntime()) {
      setRemoteStatusMessage('원격 파이프라인 불러오기는 데스크톱 Tauri 런타임에서만 사용할 수 있습니다.')
      return
    }

    const request = sanitizeRemoteTarget()
    if (!request.host || !request.username || !request.password || !remotePath.trim()) {
      setRemoteStatusMessage('원격 IP, 계정 ID, PW, 파이프라인 파일 경로를 모두 입력해 주세요.')
      return
    }

    setRemoteStatusMessage(`${request.host}:${remotePath.trim()} 원격 파일을 읽는 중...`)
    setGstreamerStatus((current) => ({
      ...current,
      remote: {
        ...current.remote,
        host: request.host,
        message: '원격 파일 읽는 중...',
        port: request.port,
        state: current.remote.state === 'connected' ? 'connected' : 'checking',
      },
    }))
    try {
      const backendDocument = await loadRemotePipeline(request, remotePath.trim())
      const viewModel = await toViewModel(backendDocument)
      setPendingPreview({
        document: viewModel,
        remoteHost: request.host,
        sourceName: remotePath.trim(),
        text: viewModel.normalizedText,
      })
      setPreviewRemoteTarget(request)
      setActiveDocument(null)
      setActiveRemoteTarget(null)
      setGstreamerStatus((current) => ({
        ...current,
        remote: {
          ...current.remote,
          host: request.host,
          message:
            current.remote.state === 'connected'
              ? '원격 파일 로드됨'
              : '원격 파일 로드됨, GStreamer API 미확인',
          port: request.port,
          state: current.remote.state === 'connected' ? 'connected' : 'unknown',
        },
      }))
      setRemoteStatusMessage(
        viewModel.diagnostics.length
          ? `원격 파이프라인 미리보기를 열었습니다. 확인할 파서 진단 ${viewModel.diagnostics.length}건이 있습니다.`
          : `원격 파이프라인 미리보기를 열었습니다. 노드 ${viewModel.graph.nodes.length}개와 엣지 ${viewModel.graph.edges.length}개를 파싱했습니다.`,
      )
    } catch (error) {
      console.error(error)
      setGstreamerStatus((current) => ({
        ...current,
        remote: {
          host: request.host,
          message: '원격 파일 로드 실패',
          port: request.port,
          state: 'failed',
        },
      }))
      setRemoteStatusMessage('원격 파이프라인을 불러오지 못했습니다. 파일 경로와 권한을 확인해 주세요.')
    }
  }

  if (pendingPreview) {
    return (
      <ImportPreviewScreen
        gstreamerStatus={gstreamerStatus}
        isGenerating={isGeneratingPreview}
        isTauri={isTauriRuntime()}
        preview={pendingPreview}
        statusMessage={statusMessage}
        onBackHome={() => {
          setPendingPreview(null)
          setPreviewRemoteTarget(null)
        }}
        onConfirm={handleGeneratePreviewTopology}
        onFileSelected={handleFileSelected}
        onTextChange={(value) =>
          setPendingPreview((current) =>
            current
              ? {
                  ...current,
                  text: value,
                }
              : current,
          )
        }
      />
    )
  }

  if (!activeDocument) {
    return (
      <HomeScreen
        gstreamerStatus={gstreamerStatus}
        isTauri={isTauriRuntime()}
        pipelineText={pipelineText}
        statusMessage={statusMessage}
        remotePath={remotePath}
        remoteStatusMessage={remoteStatusMessage}
        remoteTarget={remoteTarget}
        onFileSelected={handleFileSelected}
        onLoadRemotePipeline={handleLoadRemotePipeline}
        onParseText={handleParsePastedText}
        onPipelineTextChange={setPipelineText}
        onProbeRemote={handleProbeRemote}
        onRemotePathChange={setRemotePath}
        onRemoteTargetChange={setRemoteTarget}
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
