# 예시 파이프라인

로컬 파일 열기 흐름을 테스트할 수 있는 일반 텍스트 GStreamer 파이프라인 예시입니다.

파일 구성:
- `01_videotestsrc_linear.pld`: 가장 단순한 비디오 파이프라인 예시
- `02_videotestsrc_tee_branch.pld`: `tee` 분기 구조 예시
- `03_audiotestsrc_basic.pld`: 기본 오디오 파이프라인 예시
- `04_compositor_named_pad.pld`: `mix.sink_1` 형태의 named pad 예시
- `26_release_record_smoothing.pld`: 대형 실사용 Pipeline 회귀 확인용 예시
- `27_pipmux.pld`: 대형 실사용 Pipeline 회귀 확인용 예시
- `playback_rtsp_single_video.pld`: RTSP 영상 1개 Playback 준비 테스트 예시
- `playback_rtp_single_video.pld`: RTP/UDP 영상 1개 Playback 준비 테스트 예시
- `playback_rtp_dual_video.pld`: RTP/UDP 영상 2개 Preview split 테스트 예시
- `playback_audio_only.pld`: RTP/UDP 오디오 Playback 준비 테스트 예시
- `playback_non_streaming_blocked.pld`: Playback 차단 경로 테스트 예시
- `playback_injection_blocked.pld`: shell injection-like 문자열 무해화 테스트 예시

앱에서 `로컬 파이프라인 열기`를 누른 뒤 `fixtures/pipelines/` 폴더로 이동해서 열어보면 됩니다.
