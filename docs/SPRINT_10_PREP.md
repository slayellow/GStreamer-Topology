# Sprint 10 Prep

## Context

Sprint 10 starts from `main` after Sprint 09 was merged.

Sprint 09 delivered:
- RTP-only Playback MVP
- Remote/local playback process planning
- App preview via JPEG frame polling
- Inspector Pad Template caps visualization
- Windows no-console command execution hardening
- Windows/Linux release workflow verification

Sprint 10 should keep the same process:
- Use `GStreamer Topology Sprint Board` as the parent board.
- Use the sprint-specific `Sprint 10` GitHub Project for active execution.
- Keep work on the `sprint_10` branch.
- Record Atlas, Forge, Loom, and Beacon outputs as Korean issue comments.
- Do not close the sprint until user QA and Desktop Release verification are complete.

## GitHub Project

- Parent board: https://github.com/users/slayellow/projects/1
- Sprint board: https://github.com/users/slayellow/projects/8
- Sprint branch: `sprint_10`

## Initial Scope

### #48 Remote RTP Playback Failure With H264 Vendor Pipeline

Issue:
- https://github.com/slayellow/GStreamer-Topology/issues/48

Priority:
- P1

Type:
- Bug / QA follow-up

Observed user QA:
- Windows 11 App can connect to the remote target and detect the remote
  GStreamer version.
- Remote playback works for a simple RAW RTP sender:

```text
videotestsrc is-live=true pattern=smpte ! video/x-raw,format=RGB,width=640,height=360,framerate=30/1 ! videoconvert ! rtpvrawpay pt=96 ! application/x-rtp,media=(string)video,clock-rate=(int)90000,encoding-name=(string)RAW,sampling=(string)RGB,depth=(string)8,width=(string)640,height=(string)360,colorimetry=(string)SMPTE240M,payload=(int)96,a-framerate=(string)30 ! udpsink host=192.168.100.112 port=15000
```

- Remote playback fails for a Qualcomm camera/H264 RTP sender:

```text
qtiqmmfsrc camera=0 name=eocam0 ! video/x-raw(memory:GBM),format=NV12,framerate=30/1,width=1920,height=1080 !
qtic2venc name=eo_venc control-rate=2 idr-interval=60 target-bitrate=3000000 !
h264parse config-interval=-1 ! tee name=eoenc ! rtph264pay mtu=1350 config-interval=1 name=pay0 pt=96 ! udpsink host=192.168.100.112 port=15000
```

Likely investigation areas:
- H264 RTP caps are not rich enough for the generated local preview receiver.
- Local PC may not have the required H264 decoder plugin, such as `avdec_h264`.
- Remote process may start but local preview may not receive frames.
- `udpsink host` may need to match the Windows PC IP reachable from OE-Linux.
- Current diagnostics may collapse distinct failures into a generic playback error.

Acceptance criteria:
- The app distinguishes remote launch failure, local decoder/plugin failure,
  frame timeout, and host/IP mismatch as separate user-facing states where
  possible.
- H264 RTP preview works when the local environment has the required GStreamer
  plugins.
- If H264 preview cannot run, the user sees a concrete next action.
- Stop/close cleans up local and remote playback processes.

Verification:
- `git diff --check`
- `npm run lint`
- `npm run build`
- `cd src-tauri && cargo test`
- Remote Windows/OE-Linux user QA with both pipelines from issue #48.

### #46 RTP Playback Preview Quality Improvement Spike

Issue:
- https://github.com/slayellow/GStreamer-Topology/issues/46

Priority:
- P2

Type:
- Technical spike / performance improvement

Goal:
- Decide whether the current JPEG frame polling bridge should be replaced or
  improved.

Candidate approaches:
- `MJPEG local HTTP`
- Rust GStreamer `appsink`
- WebRTC

Constraints:
- Must preserve Windows/Linux/macOS app direction.
- Must not require executing arbitrary PLD text through a shell.
- Must not leave orphan processes or local servers after Stop/Close.
- Should not expand Sprint 10 beyond the #48 P1 failure unless time allows.

Acceptance criteria:
- Compare candidates by implementation complexity, preview smoothness, CPU cost,
  packaging risk, and Stop/cleanup behavior.
- Validate at least one candidate with a small command-level or prototype proof.
- Recommend one implementation path for a future sprint.

## Initial Sprint Order

1. Reproduce or reason through #48 from the current backend playback plan.
2. Improve diagnostics and H264 RTP receiver generation.
3. Verify local RAW RTP and H264 RTP preview paths with fixtures or command-level tests.
4. Hand #48 to user QA before deep work on #46.
5. Run #46 as a bounded spike only after #48 has a usable fix or clear blocker.

## Expert Loop Expectations

For each implementation slice:
- Atlas defines the smallest testable outcome.
- Forge implements only the approved slice.
- Loom reviews user-visible state, error messaging, and preview layout.
- Beacon verifies happy path, failure path, and regression coverage.

Post results back to the related GitHub issue in Korean before requesting user QA.

## Known Risks

- The local Windows environment may not include H264 decoder plugins by default.
- OE-Linux device-specific plugins are not reproducible on the local Mac.
- RTP H264 caps may require SPS/PPS or payload details not present in the static PLD.
- Network routing between OE-Linux and the Windows PC may be the real failure even if both GStreamer pipelines are valid.

## Handoff

Sprint 10 is prepared but implementation has not started.

Ready state:
- `Sprint 10` GitHub Project exists and is linked to `GStreamer-Topology`.
- Issues #48 and #46 are in both the parent board and Sprint 10 board as `Todo`.
- Sprint labels and role labels are applied.
- Local branch `sprint_10` is created from updated `main`.
