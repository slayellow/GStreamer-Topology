# Sprint 02 Prep

## Sprint Goal

로컬 MVP의 신뢰도를 높인다.

이번 스프린트는 새 기능 확장보다 사용자가 직접 확인한 QA 이슈를 해결해서
예시 pipeline, parser topology, canvas usability를 안정화하는 데 집중한다.

## Scope

In scope:
- `fixtures/pipelines/02_videotestsrc_tee_branch.pld` 분기 렌더링 수정
- `fixtures/pipelines/04_compositor_named_pad.pld` named pad / compositor 렌더링 수정
- advertised fixture regression test 추가
- workspace page scroll 제거 또는 최소화
- minimap contrast 개선
- 첫 화면 layout ratio 보정

Out of scope:
- 원격 `OE-Linux` 접속 기능 확장
- remote file browser
- remote `gst-inspect` metadata enrichment
- export 기능
- packaging/signing

## Candidate Issues

### 1. Fix tee inline reference branch parsing for fixture 02

Type:
- `Bug`

Priority:
- `P1`

Area:
- `Parser`

Problem:
- `fixtures/pipelines/02_videotestsrc_tee_branch.pld`에서 `tee name=t t.`와 뒤쪽
  `t.`가 element loose token으로 처리된다.
- 이 때문에 분기 구조가 기대처럼 보이지 않는다.

Acceptance Criteria:
- `tee name=t` 노드가 canvas에 표시된다.
- `t.` reference가 `tee`의 outgoing branch로 해석된다.
- `tee`와 `autovideosink`에 대해 `t.` unparsed-token warning이 발생하지 않는다.
- source span과 diagnostics 흐름은 유지된다.

Verification:
- `cargo test`
- `npm run lint`
- `npm run build`
- 수동 QA: fixture 02를 열고 `tee`에서 두 branch가 나뉘는지 확인

### 2. Fix compositor named-pad parsing for fixture 04

Type:
- `Bug`

Priority:
- `P1`

Area:
- `Parser`

Problem:
- `fixtures/pipelines/04_compositor_named_pad.pld`에서 `compositor name=mix` 노드가
  canvas에 보이지 않는다.
- compact multi-chain과 `mix.` / `mix.sink_1` 패턴 처리가 부족하다.

Acceptance Criteria:
- `compositor name=mix` 노드가 canvas에 표시된다.
- `videotestsrc pattern=smpte` branch가 `mix.`로 연결된다.
- `videotestsrc pattern=ball` branch가 `mix.sink_1`로 연결된다.
- 가짜 `mix.` node를 만들지 않는다.

Verification:
- advertised fixture 04 parser test 추가
- `cargo test`
- 수동 QA: fixture 04를 열고 compositor node와 incoming branches 확인

### 3. Make workspace fit native window without page scrolling

Type:
- `UX`

Priority:
- `P1`

Area:
- `Canvas`

Problem:
- 워크스페이스 화면에서 page scroll이 필요해 topology tool로 쓰기 불편하다.

Acceptance Criteria:
- 기본 Tauri 창 `1440x960`에서 topbar, canvas, diagnostics toggle, inspector가
  page scroll 없이 보인다.
- graph pan/zoom은 React Flow 내부에서 동작한다.
- inspector와 diagnostics의 긴 내용은 해당 panel 내부에서만 scroll된다.

Verification:
- `npm run lint`
- `npm run build`
- 수동 QA: fixture 02, 04, 26, 27 열기

### 4. Fix minimap contrast

Type:
- `UX`

Priority:
- `P2`

Area:
- `Canvas`

Problem:
- minimap이 표시되지만 내부 topology가 하얗게 보여 식별하기 어렵다.

Acceptance Criteria:
- minimap에서 node silhouette와 viewport가 명확히 보인다.
- light canvas background에서도 contrast가 충분하다.
- 작은 desktop window에서도 minimap이 식별 가능하다.

Verification:
- fixture 02와 대형 canonical sample을 열어 minimap 확인

### 5. Rebalance first screen layout ratio

Type:
- `UX`

Priority:
- `P2`

Area:
- `Import`

Problem:
- 첫 GUI 화면의 내용 비율이 전체 화면과 잘 맞지 않는다.

Acceptance Criteria:
- 첫 화면이 기본 Tauri 창에서 어색한 빈 공간이나 과도한 card 느낌 없이 보인다.
- `GStreamer Topology`, 파일 열기, 붙여넣기 영역, 예시 파일 안내가 자연스럽게 배치된다.
- 로컬 파일 열기와 붙여넣기 경로는 그대로 동작한다.

Verification:
- 기본 Tauri 창과 작은 desktop window에서 수동 확인
- local file open / paste path regression 확인

### 6. Add advertised fixture regression coverage

Type:
- `QA`

Priority:
- `P2`

Area:
- `Parser`

Problem:
- `cargo test`가 통과해도 advertised fixtures `01`-`04`의 핵심 topology regression을
  놓칠 수 있다.

Acceptance Criteria:
- `fixtures/pipelines/01_videotestsrc_linear.pld` 핵심 node/edge expectation 추가
- `fixtures/pipelines/02_videotestsrc_tee_branch.pld` branch expectation 추가
- `fixtures/pipelines/03_audiotestsrc_basic.pld` 핵심 node/edge expectation 추가
- `fixtures/pipelines/04_compositor_named_pad.pld` compositor/named-pad expectation 추가

Verification:
- `cargo test`

## Recommended Sprint Order

1. `Fix tee inline reference branch parsing for fixture 02`
2. `Fix compositor named-pad parsing for fixture 04`
3. `Add advertised fixture regression coverage`
4. `Make workspace fit native window without page scrolling`
5. `Fix minimap contrast`
6. `Rebalance first screen layout ratio`

Parser work should come first because fixtures 02 and 04 are currently advertised examples.
Viewport work should follow because it affects every future manual QA pass.

## GitHub Project Status

Project:
- `GStreamer Topology Sprint Board`
- URL: `https://github.com/users/slayellow/projects/1`
- Number: `1`

Current state:
- Project exists
- Project has default fields
- No issues are currently registered in the repo

Recommended next cloud actions:
- Create labels: `p0`, `p1`, `p2`, `parser`, `canvas`, `ux`, `qa`, `sprint`
- Create the 6 candidate issues above
- Add the issues to Project `#1`
- Move all 6 issues to `Ready for Planning` or `Todo`

These actions modify GitHub cloud data and should be confirmed before execution.
