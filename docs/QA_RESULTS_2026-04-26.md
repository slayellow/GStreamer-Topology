# QA Results - 2026-04-26

이 문서는 사용자가 직접 수행한 MVP QA 체크리스트 결과를 다음 스프린트 입력으로
정리한 기록이다.

## 통과 항목

- 자동 체크 4개 항목 통과
- 앱 실행 4개 항목 통과
- 첫 화면 UI 4개 항목 통과
- 로컬 파일 열기 5개 항목 통과
- 붙여넣기 경로 3개 항목 통과
- 인스펙터 6개 항목 통과
- 진단 패널 5개 항목 통과
- 허용 실패 경로 4개 항목 통과
- 한국어화 확인 5개 항목 통과
- 회귀 확인 3개 항목 통과

## 발견된 이슈

### QA-001: `tee` 분기 예시가 기대한 branch topology로 보이지 않음

파일:
- `fixtures/pipelines/02_videotestsrc_tee_branch.pld`

관찰 결과:
- 분기 구조 확인 실패
- 다음 경고 발생:
  - 요소 `tee` 에서 해석하지 못한 토큰: `t.`
  - 요소 `autovideosink` 에서 해석하지 못한 토큰: `t.`

초기 판단:
- parser가 `tee name=t t. ! ... t. ! ...` 형태의 inline reference branch를
  독립적인 reference endpoint로 분리하지 못하고 element loose token으로 처리한다.

우선순위:
- `P1`

영역:
- `Parser`

### QA-002: `compositor` named-pad 예시에서 compositor element가 보이지 않음

파일:
- `fixtures/pipelines/04_compositor_named_pad.pld`

관찰 결과:
- `compositor` element가 canvas에 보이지 않음

초기 판단:
- compact multi-chain pipeline과 `mix.` / `mix.sink_1` named pad 패턴을 parser가
  안정적으로 statement/edge로 모델링하지 못하고 있다.

우선순위:
- `P1`

영역:
- `Parser`

### QA-003: Minimap이 보이지만 내용이 식별되지 않음

관찰 결과:
- 미니맵은 우측 하단에 표시됨
- 미니맵 내부가 하얗게 보여 topology를 식별하기 어려움

초기 판단:
- React Flow minimap node color, mask, background contrast 조정이 필요하다.

우선순위:
- `P2`

영역:
- `Canvas`

### QA-004: 첫 GUI 화면의 내용 비율이 전체 화면에 맞지 않음

관찰 결과:
- 첫 화면에서 전체 화면 대비 내용 화면의 비율이 어색함

초기 판단:
- launcher card 폭, paste area 높이, vertical centering, 주변 여백의 균형을 다시 잡아야 한다.

우선순위:
- `P2`

영역:
- `Import`

### QA-005: 워크스페이스에서 스크롤이 필요함

관찰 결과:
- 워크스페이스 화면에서 스크롤해야 하는 부분이 있어 불편할 수 있음
- 사용자는 스크롤 없이 전체 화면이 보이는 구성을 선호함

초기 판단:
- page-level scroll 대신 canvas/inspector/diagnostics 내부 scroll로 구조를 바꾸는
  viewport fit 작업이 필요하다.

우선순위:
- `P1`

영역:
- `Canvas`

## 다음 스프린트 방향

다음 스프린트는 원격 기능으로 넘어가기 전에 local MVP hardening에 집중한다.

권장 순서:
- `P1` parser correctness: `tee` inline reference branch 처리
- `P1` parser correctness: `compositor` named pad / compact multi-chain 처리
- `P1` workspace viewport fit: page scroll 제거
- `P2` minimap contrast 개선
- `P2` first screen layout balance 개선
- `P2` fixture regression coverage 추가

## 검증 원칙

- parser 수정은 `cargo test`에 advertised fixtures `01`-`04` 회귀 테스트를 추가한다.
- visual/layout 수정은 `npm run lint`, `npm run build`, 수동 Tauri QA를 같이 수행한다.
- `npm run tauri:dev`는 네이티브 창이 실제로 뜨거나 native process가 살아 있음을
  확인해야 검증 완료로 본다.
