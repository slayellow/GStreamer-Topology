import type { ChangeEvent } from 'react'
import { Icon } from '../../components/Icon.tsx'
import { IconButton } from '../../components/IconButton.tsx'
import { ConnectionBadge } from '../../components/ConnectionBadge.tsx'
import type { GStreamerRuntimeStatus } from '../../app/status.ts'
import type { PipelineDocumentViewModel } from '../../graph/types.ts'

export type ImportPreviewViewModel = {
  document: PipelineDocumentViewModel
  remoteHost?: string
  sourceName: string
  text: string
}

type ImportPreviewScreenProps = {
  gstreamerStatus: GStreamerRuntimeStatus
  isGenerating: boolean
  isTauri: boolean
  onBackHome: () => void
  onConfirm: () => void
  onFileSelected: (event: ChangeEvent<HTMLInputElement>) => void
  onTextChange: (value: string) => void
  preview: ImportPreviewViewModel
  statusMessage: string
}

function ImportPreviewScreen({
  gstreamerStatus,
  isGenerating,
  isTauri,
  onBackHome,
  onConfirm,
  onFileSelected,
  onTextChange,
  preview,
  statusMessage,
}: ImportPreviewScreenProps) {
  const diagnostics = preview.document.diagnostics
  const syntaxDiagnostics = diagnostics.filter((diagnostic) => diagnostic.severity !== 'info')

  return (
    <main className="app-shell app-shell--home">
      <section className="import-preview panel">
        <header className="import-preview__topbar">
          <div className="import-preview__nav">
            <IconButton icon="arrowLeft" label="첫 화면으로 돌아가기" onClick={onBackHome} />
            <label
              aria-label="다른 파일 열기"
              className="icon-button icon-upload-button"
              htmlFor="preview-pipeline-file"
              title="다른 파일 열기"
            >
              <Icon name="folderOpen" />
              <span className="sr-only">다른 파일 열기</span>
              <input
                id="preview-pipeline-file"
                onChange={onFileSelected}
                type="file"
                accept=".pld,.txt"
              />
            </label>
          </div>

          <div className="import-preview__status">
            <ConnectionBadge label="GStreamer API" status={gstreamerStatus.local} />
            {preview.remoteHost ? (
              <ConnectionBadge label="Remote Server" status={gstreamerStatus.remote} />
            ) : null}
            <span className="card-chip muted-chip">
              {isTauri ? '데스크톱 런타임' : '브라우저 미리보기'}
            </span>
          </div>
        </header>

        <section className="import-preview__hero">
          <div>
            <div className="eyebrow">파일 미리보기</div>
            <h1>{preview.sourceName}</h1>
            <p className="hero-body">
              토폴로지를 생성하기 전에 파이프라인 원문과 감지된 문제를 확인하세요.
              필요한 경우 텍스트를 수정한 뒤 `토폴로지 생성`을 누르면 수정본 기준으로
              캔버스를 엽니다.
            </p>
          </div>

          <div className="import-preview__summary">
            <span className="card-chip">
              노드 {preview.document.graph.nodes.length}개 / 엣지 {preview.document.graph.edges.length}개
            </span>
            <span className={syntaxDiagnostics.length ? 'card-chip severity-warning' : 'card-chip'}>
              진단 {diagnostics.length}개
            </span>
          </div>
        </section>

        {syntaxDiagnostics.length ? (
          <aside className="workspace-alert severity-warning">
            <strong>파이프라인 구문 확인 필요</strong>
            <span>
              가능한 범위의 토폴로지는 만들 수 있지만, 파서 진단
              {syntaxDiagnostics.length}건이 있습니다. 생성 전 원문을 확인해 주세요.
            </span>
          </aside>
        ) : null}

        <textarea
          className="text-area import-preview__editor"
          onChange={(event) => onTextChange(event.target.value)}
          spellCheck={false}
          value={preview.text}
        />

        <footer className="import-preview__footer">
          <span className="status-message">{statusMessage}</span>
          <button
            className="primary-button"
            disabled={isGenerating || !preview.text.trim()}
            onClick={onConfirm}
            type="button"
          >
            {isGenerating ? '생성 중...' : '토폴로지 생성'}
          </button>
        </footer>
      </section>
    </main>
  )
}

export { ImportPreviewScreen }
