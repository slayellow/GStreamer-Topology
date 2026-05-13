# Sprint 09 Preparation

Date: 2026-05-13

Branch:
- `sprint_09`

Base:
- `main` after Sprint 08 merge commit `51b13b5`

## Sprint Setup Status

GitHub Project note:
- Parent board: `GStreamer Topology Sprint Board`
- Sprint execution board: `Sprint 09`
- Project URL: `https://github.com/users/slayellow/projects/7`
- The `Sprint 09` Project is linked to the `GStreamer-Topology` repository.
- Status field uses `Todo`, `In Progress`, and `Done`.
- Sprint label: `sprint-09`

Sprint board items:
- `#43` `스프린트 09: Windows 첫 실행 Canvas 지연/명령 프롬프트 노출 문제`
- `#44` `스프린트 09: Inspector Pad Templates 상세 Caps 정보 시각화`
- `#45` `스프린트 09: RTP/RTSP Pipeline Playback 제어 창 MVP`

Setup completed:
- `#43`, `#44`, and `#45` are on the parent board.
- `#43`, `#44`, and `#45` are on the `Sprint 09` Project.
- `#43`, `#44`, and `#45` have the `sprint-09` label.
- `#43`, `#44`, and `#45` are currently `Todo`.
- All sprint issues include role responsibilities, acceptance criteria, and
  user QA checklists in Korean.

## Expected Sprint Theme

Sprint 09 should focus on Windows first-run reliability, richer Inspector
metadata, and a bounded RTP/RTSP Playback MVP.

The sprint should answer these practical questions:
- Can the Windows installed app avoid first-run Canvas/source-click freezes?
- Can the Windows installed app avoid command prompt windows during first-run
  command probes or metadata lookups?
- Can Inspector Pad Templates show useful caps/format details from
  `gst-inspect-1.0` output without breaking layout?
- Can the same Inspector metadata structure work for Local and Remote authority
  paths where GStreamer API data is available?
- Can the app detect RTP/RTSP endpoints with IP/Port from PLD text and control a
  local GStreamer playback process safely?
- Can Playback be kept distinct from Simulation so runtime process state does
  not affect static validation or topology analysis?

## Issue Triage

### `#43` Windows first-run Canvas/source-click freeze

Priority:
- `P1`

User evidence:
- The issue appears after installing the Windows Release and running the app for
  the first time.
- Canvas Element random clicks and Pipeline Source element clicks can freeze the
  UI.
- A command prompt appears briefly and disappears before the UI continues.
- The second app launch behaves normally.
- App launch, topology generation, and remote simulation were reported as
  normal.

Sprint 09 target outcome:
- Identify whether first-run `gst-inspect-1.0`, `gst-launch-1.0`, shell command
  spawn, cache warm-up, or metadata probe paths cause the freeze and command
  prompt.
- Prevent visible command prompt windows in packaged Windows flows.
- Keep first-run Canvas selection/source navigation responsive.
- Preserve Local/Remote Simulation behavior.

### `#44` Inspector Pad Templates detailed caps display

Priority:
- `P2`

User evidence:
- Inspector currently shows Pad Templates too simply.
- The user needs to see what format/caps can be used on Element input/output
  pads.

Sprint 09 target outcome:
- Extend `gst-inspect-1.0` metadata parsing to collect Pad Templates caps/detail
  text where available.
- Show pad name, direction, presence, and caps/format details in Inspector.
- Keep long caps strings readable with wrapping or compact structured sections.
- Preserve graceful fallback when Local/Remote GStreamer API is unavailable or
  the output cannot be parsed.

### `#45` RTP/RTSP Pipeline Playback control window MVP

Priority:
- `P1`

User request:
- Add a Playback icon on the topology screen.
- Clicking the icon opens a separate playback window or large panel.
- The window provides `Pipeline 재생 준비`, `재생`, and `중지` controls.
- Prepare analyzes the current PLD source and only enables playback for RTP or
  RTSP streams that include IP/Port information.
- Prepare determines whether the PLD describes one video/audio stream or
  multiple streams, then splits the preview area into matching slots.
- Play runs a local GStreamer playback pipeline for the detected stream.
- Stop terminates the running playback pipeline.
- The window shows the original PLD source and the generated playback pipeline.

Sprint 09 target outcome:
- Implement RTP/RTSP endpoint detection for `rtsp://IP:PORT/...` and common
  `udpsrc ... port=... application/x-rtp` patterns.
- Add a Playback UI with clear states: `Idle`, `Prepared`, `Playing`,
  `Stopped`, and `Error`.
- Add safe local process lifecycle commands for prepare, start, stop, and
  status.
- Prevent duplicate Play processes and clean up a running process when the
  playback window or app closes.
- Block playback before execution when local GStreamer is unavailable or no
  RTP/RTSP endpoint with IP/Port is detected.
- Keep Playback separate from Simulation. Simulation remains a short validation
  path; Playback is a long-running process control path.

Explicitly out of scope for Sprint 09:
- Perfectly converting every arbitrary PLD into a playable pipeline.
- Fully embedding all GStreamer video/audio sinks inside the Tauri WebView.
- Running playback on the remote OE-Linux target.
- HLS, WebRTC, MJPEG, or Rust GStreamer `appsink` preview architecture unless a
  separate spike proves the path is safe enough.

Key risk:
- `gst-launch-1.0 ... autovideosink` normally uses an OS-native video output and
  does not automatically render inside a WebView `<video>` element. The first
  MVP should stabilize stream detection, preview slot preparation, and process
  control. True in-app media rendering should be treated as a follow-up spike if
  it requires platform-specific sink or frame-bridge work.

## Recommended Implementation Order

1. `#43` Windows first-run reliability.
   - This is the highest-risk field issue because it affects trust in the
     installed Windows build and can only be fully verified through packaged
     Windows QA.
2. `#45` RTP/RTSP Playback MVP.
   - This is a high-risk new workflow because it starts long-running local
     processes and overlaps with the Windows console-window issue in `#43`.
     Keep the slice small and block unsafe or unsupported inputs early.
3. `#44` Inspector Pad Templates detail.
   - This is a contained metadata and UI enhancement and should follow after
     the Windows first-run subprocess path is understood, because both may touch
     metadata command execution.

## Expert Loop Plan

For each substantial implementation slice, use the established role contract:

Atlas:
- Restate the user-visible goal.
- Keep the slice small.
- Define acceptance criteria, verification steps, and stop conditions.

Forge:
- Implement only the active issue.
- Avoid broad refactors.
- Preserve tolerant behavior and diagnostics.
- Keep remote behavior read-only in MVP.

Loom:
- Check first-run state feedback and Inspector readability.
- Preserve the Miro-inspired Technical Canvas direction.
- Avoid visual changes that increase large-graph render cost.

Beacon:
- Verify the slice against issue criteria.
- Record internal QA results as a Korean comment on the matching GitHub issue.
- Explicitly state what remains unverified for user QA.

## Verification Standard

Before handing Sprint 09 work to the user for QA, run the relevant subset of:
- `git diff --check`
- `npm run lint`
- `npm run build`
- `cd src-tauri && cargo test`
- `npm run tauri:build`

For Windows first-run work:
- Verify code-level subprocess behavior on macOS/Linux where possible.
- Use Windows-specific spawn flags or platform behavior checks where applicable.
- Trigger the `Desktop Release` workflow before closeout and confirm Windows
  installer assets plus checksum files.
- Report Windows first-run installed-app behavior as user-QA-required unless it
  is directly verified on a Windows machine.

For Inspector Pad Templates work:
- Verify a Local sample element such as `videotestsrc`.
- Verify an element with richer caps such as `videoconvert`.
- Verify fallback behavior when metadata is unavailable.
- If Remote OE-Linux is not available to the agent, report Remote Pad Templates
  enrichment as user-QA-required.

For Playback MVP work:
- Verify RTP/RTSP endpoint detection with fixture-level tests.
- Verify a non-streaming fixture is blocked before Play.
- Verify shell injection-like text is not passed through a shell.
- Verify Play cannot create duplicate child processes.
- Verify Stop terminates the active child process and clears UI state.
- Verify Simulation still uses its short validation path and does not share
  Playback process state.
- If a real RTP/RTSP stream is not available to the agent, report live media
  playback as user-QA-required.

## Handoff Expectation

At every Sprint 09 handoff, report:
- which issue was worked
- what changed
- what internal checks passed
- what remains unverified
- which issue needs user QA next
