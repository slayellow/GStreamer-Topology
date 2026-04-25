import { useState, type ChangeEvent } from 'react'
import {
  isTauriRuntime,
  parsePipelineText,
} from './backend.ts'
import { HomeScreen } from '../features/home/HomeScreen.tsx'
import { WorkspaceShell } from '../features/workspace/WorkspaceShell.tsx'
import { toViewModel } from '../graph/fromBackend.ts'
import type { PipelineDocumentViewModel } from '../graph/types.ts'

function AppShell() {
  const [activeDocument, setActiveDocument] = useState<PipelineDocumentViewModel | null>(null)
  const [pipelineText, setPipelineText] = useState('')
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
      setStatusMessage(
        `${sourceName}에서 노드 ${viewModel.graph.nodes.length}개와 엣지 ${viewModel.graph.edges.length}개를 파싱했습니다.`,
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

  if (!activeDocument) {
    return (
      <HomeScreen
        isTauri={isTauriRuntime()}
        pipelineText={pipelineText}
        statusMessage={statusMessage}
        onFileSelected={handleFileSelected}
        onParseText={handleParsePastedText}
        onPipelineTextChange={setPipelineText}
      />
    )
  }

  return (
    <WorkspaceShell
      key={activeDocument.id}
      document={activeDocument}
      onBackHome={() => setActiveDocument(null)}
    />
  )
}

export default AppShell
