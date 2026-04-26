# GStreamer Topology

GStreamer pipeline text를 로컬에서 불러와 topology canvas로 시각화하는
크로스플랫폼 데스크톱 앱입니다.

현재 앱은 `.pld`, `.txt` 파일 또는 붙여넣은 pipeline text를
읽고, Rust parser가 만든 graph IR을 `React Flow + ELK` 기반 캔버스에 렌더링합니다.

## 현재 상태

구현됨:
- `Tauri 2 + Rust + React + TypeScript` 기반 데스크톱 앱
- 로컬 파일 열기와 붙여넣기 파싱
- plain pipeline text 정규화
- tolerant pipeline parser
- topology canvas, minimap, zoom/pan controls
- 노드 선택 기반 inspector
- 접이식 diagnostics panel
- PNG/JPG topology export
- macOS/Windows release build workflow
- 한국어 UI chrome
- 테스트용 예시 pipeline 파일

아직 진행 중:
- 실제 `OE-Linux` 장비 대상 SSH/SFTP 검증
- 원격 파일 브라우저
- remote `gst-inspect-1.0` 기반 element metadata enrichment
- search, release signing/notarization

## 빠른 실행

의존성을 설치합니다.

```bash
npm install
```

데스크톱 앱을 실행합니다.

```bash
npm run tauri:dev
```

첫 실행은 Rust/Tauri 컴파일 때문에 시간이 걸릴 수 있습니다.

## 검증 명령

```bash
npm run lint
npm run build
cd src-tauri && cargo test
```

`npm run tauri:dev`는 Vite 로그만 보고 성공으로 판단하지 않습니다. 네이티브 창이
실제로 열리거나 native process가 살아 있음을 확인해야 합니다.

## 설치 파일 만들기

로컬 macOS 개발 환경에서 설치 가능한 번들을 만들려면 아래 명령을 실행합니다.

```bash
npm run tauri:build
```

생성물은 `src-tauri/target/release/bundle/` 아래에 만들어집니다. macOS에서는
보통 `.app`과 `.dmg`가 생성됩니다.

Windows/macOS 배포 파일은 GitHub Actions의 `Desktop Release` workflow로 만들 수
있습니다.

1. GitHub 저장소의 `Actions` 탭에서 `Desktop Release`를 선택합니다.
2. `Run workflow`를 누르고 `app-v0.1.0` 같은 tag를 입력합니다.
3. workflow가 끝나면 draft GitHub Release에 Windows/macOS artifact가 첨부됩니다.

현재 Release artifact는 unsigned/not notarized 상태입니다. 공개 배포 전에는
Windows code signing과 macOS notarization을 별도 스프린트에서 다루는 것을
권장합니다.

Known limitation:
- macOS에서 `.dmg`로 설치한 앱을 Finder에서 실행하면 interactive shell `PATH`를
  상속하지 않습니다. `gst-inspect-1.0`이 Anaconda, Homebrew처럼 shell profile에만
  등록된 경로에 있으면 앱의 `GStreamer API` 상태가 unavailable로 보일 수
  있습니다. 다음 스프린트에서는 common path probe와 사용자 지정 binary path
  설정으로 보완할 예정입니다.

## 예시 파이프라인

앱의 `로컬 파이프라인 열기`에서 아래 파일을 열어볼 수 있습니다.

- [fixtures/pipelines/01_videotestsrc_linear.pld](fixtures/pipelines/01_videotestsrc_linear.pld)
- [fixtures/pipelines/02_videotestsrc_tee_branch.pld](fixtures/pipelines/02_videotestsrc_tee_branch.pld)
- [fixtures/pipelines/03_audiotestsrc_basic.pld](fixtures/pipelines/03_audiotestsrc_basic.pld)
- [fixtures/pipelines/04_compositor_named_pad.pld](fixtures/pipelines/04_compositor_named_pad.pld)
- [fixtures/pipelines/26_release_record_smoothing.pld](fixtures/pipelines/26_release_record_smoothing.pld)
- [fixtures/pipelines/27_pipmux.pld](fixtures/pipelines/27_pipmux.pld)

붙여넣기 테스트용:

```text
videotestsrc pattern=smpte ! videoconvert ! autovideosink
```

## 주요 문서

- [Agent handoff](AGENTS.md)
- [Process policy](docs/PROCESS_POLICY.md)
- [Product requirements](docs/PRD.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Implementation plan](docs/IMPLEMENTATION_PLAN.md)
- [GitHub Projects 운영안](docs/GITHUB_PROJECTS.md)

## 개발 원칙

- 로컬 앱 대상 플랫폼: `Windows`, `Linux`, `macOS`
- 원격 대상 장비: `OE-Linux`
- 원격 기능은 MVP에서 read-only로 시작
- local GStreamer plugin 설치에 의존하지 않는 parser-first 구조
- 큰 기능은 `Planner -> Developer -> Designer -> QA` 흐름으로 진행
- QA evidence 없이 완료 처리하지 않음

## GitHub 운영

이 repo는 GitHub Issues와 GitHub Projects로 스프린트를 관리할 수 있게 구성되어
있습니다.

Issue templates:
- `Feature Request`
- `Bug Report`
- `Sprint Task`
- `QA Report`

권장 칸반 보드 구조와 Project 생성 방법은
[docs/GITHUB_PROJECTS.md](docs/GITHUB_PROJECTS.md)를 참고하세요.

## 라이선스

아직 라이선스 파일은 추가하지 않았습니다. 공개 배포 전 라이선스 결정을 권장합니다.
