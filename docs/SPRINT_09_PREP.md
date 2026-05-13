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

Setup completed:
- `#43` and `#44` are on the parent board.
- `#43` and `#44` are on the `Sprint 09` Project.
- `#43` and `#44` have the `sprint-09` label.
- `#43` and `#44` are currently `Todo`.
- Both issues include role responsibilities, acceptance criteria, and user QA
  checklists in Korean.

## Expected Sprint Theme

Sprint 09 should focus on Windows first-run reliability and richer Inspector
metadata.

The sprint should answer these practical questions:
- Can the Windows installed app avoid first-run Canvas/source-click freezes?
- Can the Windows installed app avoid command prompt windows during first-run
  command probes or metadata lookups?
- Can Inspector Pad Templates show useful caps/format details from
  `gst-inspect-1.0` output without breaking layout?
- Can the same Inspector metadata structure work for Local and Remote authority
  paths where GStreamer API data is available?

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

## Recommended Implementation Order

1. `#43` Windows first-run reliability.
   - This is the highest-risk field issue because it affects trust in the
     installed Windows build and can only be fully verified through packaged
     Windows QA.
2. `#44` Inspector Pad Templates detail.
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

## Handoff Expectation

At every Sprint 09 handoff, report:
- which issue was worked
- what changed
- what internal checks passed
- what remains unverified
- which issue needs user QA next
