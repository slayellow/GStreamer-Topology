# 예시 파이프라인

로컬 파일 열기 흐름을 테스트할 수 있는 일반 텍스트 GStreamer 파이프라인 예시입니다.

파일 구성:
- `01_videotestsrc_linear.pld`: 가장 단순한 비디오 파이프라인 예시
- `02_videotestsrc_tee_branch.pld`: `tee` 분기 구조 예시
- `03_audiotestsrc_basic.pld`: 기본 오디오 파이프라인 예시
- `04_compositor_named_pad.pld`: `mix.sink_1` 형태의 named pad 예시

앱에서 `로컬 파이프라인 열기`를 누른 뒤 `fixtures/pipelines/` 폴더로 이동해서 열어보면 됩니다.
