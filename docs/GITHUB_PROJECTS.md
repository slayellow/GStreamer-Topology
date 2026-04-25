# GitHub Projects 운영안

이 문서는 `GStreamer Topology`를 GitHub Issues와 GitHub Projects로 운영하기 위한
칸반 보드 기준입니다.

## 권장 보드

보드 이름:
- `GStreamer Topology Sprint Board`

보드 목적:
- 요청사항을 작은 카드로 쪼개고, `기획 -> 개발 -> 디자인 -> QA -> 완료` 흐름을
  한 곳에서 추적한다.

## 컬럼

- `Inbox`: 아직 정리되지 않은 요청, 아이디어, 버그 제보
- `Ready for Planning`: 다음 스프린트 후보로 볼 수 있는 카드
- `Planned`: acceptance criteria와 검증 기준이 정리된 카드
- `In Progress`: 개발 또는 문서 작업 진행 중
- `Design Review`: UI, UX, 카피, 정보구조 검토 대기
- `QA`: 기능 검증, 회귀 확인, 수동 테스트 진행 중
- `Blocked`: 외부 입력, 환경, 권한, 장비가 필요한 카드
- `Done`: 구현과 QA evidence가 모두 완료된 카드

## 필드

- `Type`: `Feature`, `Bug`, `UX`, `QA`, `Docs`, `Research`
- `Priority`: `P0`, `P1`, `P2`, `P3`
- `Sprint`: GitHub Projects의 `Iteration` 필드 사용
- `Owner Role`: `Planner`, `Developer`, `Designer`, `QA`
- `Area`: `Parser`, `Canvas`, `Inspector`, `Import`, `Remote`, `Docs`, `Build`
- `Verification`: `Not Started`, `In Progress`, `Passed`, `Failed`, `Blocked`
- `Risk`: `Low`, `Medium`, `High`

## 카드 처리 규칙

- 모든 큰 기능은 `Planner`, `Developer`, `Designer`, `QA` 관점을 거친다.
- `Planned`로 이동하기 전에 acceptance criteria와 out-of-scope를 작성한다.
- `QA`에서 발견된 결함은 같은 카드에 체크리스트로 남기거나 별도 `Bug` 이슈로 분리한다.
- `Done`으로 이동하려면 검증 결과와 남은 미검증 범위를 명시한다.
- `tauri:dev`는 네이티브 창이 실제로 뜨거나 native process가 살아 있음을 확인해야 검증 완료로 본다.

## 스프린트 카드 기준

카드 하나는 가능한 한 하나의 사용자 가치를 담아야 한다.

좋은 카드 예시:
- `로컬 .pld 파일 열기 후 토폴로지 렌더링`
- `노드 선택 시 인스펙터 속성 표시`
- `진단 패널 한국어화`

너무 큰 카드 예시:
- `원격 기능 전체 구현`
- `UI 전부 개선`
- `파서 완성`

## GitHub Project 생성 명령

GitHub CLI의 Projects 명령은 `project` 토큰 스코프가 필요하다.

현재 계정에 스코프가 없다면 사용자가 직접 승인해야 한다.

```bash
gh auth refresh -s project
```

승인 후 아래 명령으로 보드를 만들 수 있다.

```bash
gh project create --owner slayellow --title "GStreamer Topology Sprint Board"
```

생성된 project number를 확인한다.

```bash
gh project list --owner slayellow
```

보드를 repo와 연결한다.

```bash
gh project link <PROJECT_NUMBER> --owner slayellow --repo GStreamer-Topology
```

필드는 GitHub UI에서 직접 추가하는 편이 가장 안전하다. 프로젝트를 만든 뒤
이 문서의 `컬럼`과 `필드`를 기준으로 보드를 구성한다.

## 이슈 템플릿

이 repo에는 다음 이슈 템플릿이 있다.

- `Feature Request`: 새 기능 요청
- `Bug Report`: 결함 제보
- `Sprint Task`: 기획/개발/디자인/QA가 함께 처리할 작업 단위
- `QA Report`: 테스트 결과와 재현 정보를 남기는 QA 카드

각 이슈는 생성 후 GitHub Project에 추가해서 스프린트 보드에서 추적한다.
