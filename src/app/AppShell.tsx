import { useState, type ChangeEvent } from 'react'
import {
  isTauriRuntime,
  loadRemotePipeline,
  parsePipelineText,
  probeRemoteTarget,
  type RemoteTargetInput,
} from './backend.ts'
import { HomeScreen } from '../features/home/HomeScreen.tsx'
import { WorkspaceShell } from '../features/workspace/WorkspaceShell.tsx'
import { toViewModel } from '../graph/fromBackend.ts'
import type { PipelineDocumentViewModel } from '../graph/types.ts'

function AppShell() {
  const [activeDocument, setActiveDocument] = useState<PipelineDocumentViewModel | null>(null)
  const [pipelineText, setPipelineText] = useState('')
  const [remoteTarget, setRemoteTarget] = useState<RemoteTargetInput>({
    host: '',
    port: 22,
    username: '',
    password: '',
  })
  const [remotePath, setRemotePath] = useState('')
  const [activeRemoteTarget, setActiveRemoteTarget] = useState<RemoteTargetInput | null>(null)
  const [remoteStatusMessage, setRemoteStatusMessage] = useState(
    '원격 OE-Linux 장비가 준비되면 IP, 계정, 경로를 입력해 읽기 전용으로 불러올 수 있습니다.',
  )
  const [statusMessage, setStatusMessage] = useState(
    '로컬 파이프라인 파일을 열거나 텍스트를 붙여넣어 토폴로지로 변환하세요.',
  )

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

  async function handleFileSelected(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0]
    if (!file) {
      return
    }

    const rawText = await file.text()
    await handlePipelineImport(rawText, file.name)
    event.target.value = ''
  }

  async function handleParsePastedText() {
    if (!pipelineText.trim()) {
      setStatusMessage('파싱하기 전에 파이프라인 텍스트를 붙여넣으세요.')
      return
    }

    await handlePipelineImport(pipelineText, '붙여넣은 파이프라인')
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
    try {
      const response = await probeRemoteTarget(request, 'videotestsrc')
      setRemoteStatusMessage(
        `${response.host} 접속 성공. ${response.version_output.split('\n')[0] ?? 'GStreamer 정보를 확인했습니다.'}`,
      )
    } catch (error) {
      console.error(error)
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
    try {
      const backendDocument = await loadRemotePipeline(request, remotePath.trim())
      const viewModel = await toViewModel(backendDocument)
      setActiveDocument(viewModel)
      setActiveRemoteTarget(request)
      setRemoteStatusMessage(
        viewModel.diagnostics.length
          ? `원격 파이프라인을 열었습니다. 확인할 파서 진단 ${viewModel.diagnostics.length}건이 있습니다.`
          : `원격 파이프라인을 열었습니다. 노드 ${viewModel.graph.nodes.length}개와 엣지 ${viewModel.graph.edges.length}개를 파싱했습니다.`,
      )
    } catch (error) {
      console.error(error)
      setRemoteStatusMessage('원격 파이프라인을 불러오지 못했습니다. 파일 경로와 권한을 확인해 주세요.')
    }
  }

  if (!activeDocument) {
    return (
      <HomeScreen
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
      remoteTarget={activeRemoteTarget}
      onBackHome={() => setActiveDocument(null)}
    />
  )
}

export default AppShell
