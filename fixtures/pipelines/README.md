# 예시 파이프라인

로컬 파일 열기 흐름을 테스트할 수 있는 일반 텍스트 GStreamer 파이프라인 예시입니다.

파일 구성:
- `01_videotestsrc_linear.pld`: 가장 단순한 비디오 파이프라인 예시
- `02_videotestsrc_tee_branch.pld`: `tee` 분기 구조 예시
- `03_audiotestsrc_basic.pld`: 기본 오디오 파이프라인 예시
- `04_compositor_named_pad.pld`: `mix.sink_1` 형태의 named pad 예시
- `26_release_record_smoothing.pld`: 대형 실사용 Pipeline 회귀 확인용 예시
- `27_pipmux.pld`: 대형 실사용 Pipeline 회귀 확인용 예시
- `playback_rtp_single_video.pld`: RTP/UDP 영상 1개 Playback 준비 테스트 예시
- `playback_rtp_dual_video.pld`: RTP/UDP 영상 2개 Preview split 테스트 예시
- `playback_rtp_videotestsrc_sender.pld`: `videotestsrc` 기반 RTP/UDP RAW 송신 및 Playback 감지 예시
- `playback_rtp_videotestsrc_receiver.pld`: `videotestsrc` test sender 자동 생성을 확인하는 RTP/UDP RAW 수신 예시
- `playback_rtp_videotestsrc_dual_sender.pld`: `videotestsrc` 기반 RTP/UDP 영상 2개 송신 및 Preview split 감지 예시
- `playback_audio_only.pld`: RTP/UDP 오디오 Playback 준비 테스트 예시
- `playback_non_streaming_blocked.pld`: Playback 차단 경로 테스트 예시
- `playback_injection_blocked.pld`: shell injection-like 문자열 무해화 테스트 예시

앱에서 `로컬 파이프라인 열기`를 누른 뒤 `fixtures/pipelines/` 폴더로 이동해서 열어보면 됩니다.

## Playback QA용 videotestsrc 실행 예시

RTP/UDP 1개 영상 QA:

1. 앱에서 `playback_rtp_videotestsrc_sender.pld`를 열고 토폴로지를 생성합니다.
2. Playback 아이콘을 누른 뒤 `Pipeline 재생 준비`를 누릅니다.
3. RTP 영상 1개가 감지되고, `재생` 버튼이 활성화되는지 확인합니다.
4. `재생`을 누르면 앱이 sender PLD와 미리보기 receiver Pipeline을 함께 실행합니다.

동일 Pipeline을 터미널에서 직접 확인하려면 아래 명령을 사용합니다.

```bash
gst-launch-1.0 videotestsrc is-live=true pattern=smpte ! 'video/x-raw,format=RGB,width=640,height=360,framerate=30/1' ! videoconvert ! rtpvrawpay pt=96 ! 'application/x-rtp,media=(string)video,clock-rate=(int)90000,encoding-name=(string)RAW,sampling=(string)RGB,depth=(string)8,width=(string)640,height=(string)360,colorimetry=(string)SMPTE240M,payload=(int)96,a-framerate=(string)30' ! udpsink host=127.0.0.1 port=5004
```

RTP/UDP 2개 영상 QA:

1. 앱에서 `playback_rtp_videotestsrc_dual_sender.pld`를 열고 토폴로지를 생성합니다.
2. Playback 준비 시 Preview 슬롯이 2개로 나뉘는지 확인합니다.
3. `재생`을 누르면 앱이 sender PLD와 미리보기 receiver Pipeline을 함께 실행합니다.

RTP/UDP Receiver PLD QA:

1. 앱에서 `playback_rtp_videotestsrc_receiver.pld`를 열고 토폴로지를 생성합니다.
2. Playback 준비 시 Source Role이 `Source Receiver`로 표시되는지 확인합니다.
3. `재생`을 누르면 앱이 원본 receiver PLD와 자동 생성된 local `videotestsrc` sender를 함께 실행합니다.
4. Preview 카드에 자동 생성 sender의 영상 frame이 표시되는지 확인합니다.

주의:

- `rtpvrawpay`, `udpsink` 중 하나라도 설치되어 있지 않으면 Simulation 또는 실제 송신 실행은 실패합니다.
- 이번 Playback slice는 RTP/UDP만 지원합니다. RTSP 자동 재생/서버 생성은 제외합니다.
- Remote 모드에서는 현재 Topology PLD를 Remote에서 실행하고, 반대편 Pipeline과 App preview는 Local에서 실행합니다.
- Remote Sender PLD의 `udpsink host`는 이 PC에서 수신 가능한 IP로 설정되어 있어야 Preview가 표시됩니다.
- 이 경우 앱은 실패를 정상적으로 알려주는 것이 맞고, 해당 PC의 GStreamer plugin 설치 상태를 먼저 확인해야 합니다.
- 확인 명령은 `gst-inspect-1.0 rtpvrawpay`, `gst-inspect-1.0 udpsink`입니다.
- macOS에서 Anaconda가 Homebrew보다 앞에 있으면 `/opt/homebrew/bin/gst-launch-1.0`, `/opt/homebrew/bin/gst-inspect-1.0`처럼 Homebrew 경로를 직접 지정해 확인합니다.
