# Sprint 06 Preparation

Date: 2026-04-27

Branch:
- `sprint_06`

Base:
- `main` after Sprint 05 merge commit `cf2ca7e`

## Sprint Setup Status

GitHub Project note:
- Parent board: `GStreamer Topology Sprint Board`
- Sprint execution board: `Sprint 06`
- Project URL: `https://github.com/users/slayellow/projects/4`
- The `Sprint 06` Project is linked to the `GStreamer-Topology` repository.
- Status field uses `Todo`, `In Progress`, and `Done`.
- Sprint label: `sprint-06`

Sprint board items:
- `#25` `스프린트 05: 토폴로지 Export PNG JPG 저장 기능`
- `#27` `다음 스프린트 후보: 대형 그래프 선택 노드 자동 중앙 이동 최적화`
- `#28` `스프린트 05: Windows/Mac 설치 파일 Release 패키징`
- `#30` `[Bug] Windows 11 환경에서 SW 실행 X (Error 발생)`
- `#31` `[Feature] 원격 접속 기능 확인 및 Inspector 기능 개선 요청`

Setup completed:
- All five issues are on the parent board.
- All five issues are on the `Sprint 06` Project.
- All five issues have the `sprint-06` label.
- Sprint-specific planning comments were added to each issue in Korean.
- `#25`, `#27`, `#28`, `#30`, and `#31` are ready for Sprint 06 intake.

## Expected Sprint Theme

Sprint 06 should focus on release/runtime hardening and carry-over usability
issues from Sprint 05.

The sprint should answer one practical question first:
- Can a Windows 11 user install the Release asset and open the app without
  using the CLI?

After that blocker is addressed, continue with:
- packaged GStreamer discovery on macOS/Windows
- reliable PNG/JPG export
- large-graph focus performance
- full remote Inspector property display

## Issue Triage

### `#30` Windows 11 launch failure

Priority:
- `P1`, release blocker

User evidence:
- Windows 11 Release install completed, but app launch fails before the first
  screen.
- Error mentions missing entry point `TaskDialogIndirect`.
- Screenshot shows the app was launched in or near `C:\Program Files\GStreamer`,
  so Windows DLL search path or native dialog dependency collision is a likely
  hypothesis, not a confirmed root cause.

Sprint 06 outcome:
- Windows 11 installed app opens the first screen from Start Menu, direct exe
  launch, and non-repo working directories.
- GStreamer API discovery failure must be shown inside the app, not as an OS
  startup crash.

### `#28` Windows/Mac Release packaging follow-up

Priority:
- `P1`

Known carry-over:
- GitHub Release assets are created, but packaged runtime quality is not fully
  proven.
- macOS `.dmg` launches, but GUI-launched app may not inherit shell `PATH`, so
  `gst-inspect-1.0` can be missed even when it works in Terminal.
- macOS browser-downloaded DMG can be blocked by Gatekeeper as "damaged" when
  the app is unsigned/not notarized. This blocks user QA before the app opens
  and must be treated as a release blocker, not a user workaround.

Sprint 06 outcome:
- Add deterministic local `gst-inspect-1.0` discovery for common packaged app
  launch environments.
- Separate "GStreamer not installed" from "installed but not discoverable from
  GUI PATH".
- Keep Windows launch failure linked to `#30`.
- Ship Sprint 06 official Release artifacts for Windows/Linux only.
- Exclude macOS from GitHub Release until Apple Developer Program and Developer
  ID notarization credentials are available.
- Use local `npm run tauri:build` output only for macOS internal QA on the
  development Mac. This is not a team-distribution artifact.

### `#25` PNG/JPG export carry-over

Priority:
- `P1`

Known carry-over:
- Export UI is reachable, but user QA still received `Save canceled` instead of
  a saved image file.

Sprint 06 outcome:
- PNG and JPG export must create non-empty files.
- `Save canceled` should only appear when the user really cancels.
- Write failures must show actionable failure messages.

### `#27` large graph focus optimization

Priority:
- `P2`

Known carry-over:
- Sprint 05 removed janky automatic movement, which the user accepted.
- The user still liked the idea of centering the selected node when it can be
  done smoothly.

Sprint 06 outcome:
- Keep normal click selection stable.
- Add or redesign an explicit `Focus selected` action.
- Verify against `fixtures/pipelines/26_release_record_smoothing.pld` and
  `fixtures/pipelines/27_pipmux.pld`.

### `#31` remote Inspector property completeness

Priority:
- `P2`

User evidence:
- Remote connection and remote `gst-inspect-1.0` metadata display were verified
  by the user.
- Inspector properties appear partial rather than complete.

Sprint 06 outcome:
- Improve remote `gst-inspect-1.0 <element>` output parsing and display.
- Preserve partial success: unknown or unparsed fields should not break the
  topology or Inspector.
- Use sample `gst-inspect` output fixtures internally when a real OE-Linux
  target is not available.

## Recommended Implementation Order

1. `#30` Windows 11 launch failure.
   - This blocks any Windows user QA and should be treated as the first slice.
2. `#28` packaged GStreamer discovery and release smoke path.
   - Once the app launches, make GStreamer status reliable and understandable.
3. `#25` PNG/JPG export save flow.
   - Export depends on packaged file system behavior, so test after release
     runtime assumptions are clearer.
4. `#31` remote Inspector property completeness.
   - The user has already verified remote connection basics, so this can focus
     on metadata completeness.
5. `#27` large graph focus optimization.
   - Useful UX improvement, but less blocking than launch/export/runtime
     correctness.

## Expert Loop Plan

For each substantial implementation slice, use the established role contract:

Atlas:
- Restate the user-visible goal.
- Keep the slice small.
- Define acceptance criteria, verification steps, and stop conditions.

Forge:
- Implement only the active issue.
- Avoid broad refactors.
- Preserve parser source-span mapping and partial-success diagnostics.

Loom:
- Check information hierarchy and feedback text.
- Keep the Technical Canvas direction and avoid hiding important status.

Beacon:
- Verify the slice against issue criteria.
- Record internal QA results as a Korean comment on the matching GitHub issue.
- Explicitly state what remains unverified for user QA.

## Verification Standard

Before handing Sprint 06 work to the user for QA, run the relevant subset of:
- `git diff --check`
- `npm run lint`
- `npm run build`
- `cd src-tauri && cargo test`
- `npm run tauri:build`

For release-related issues:
- Verify GitHub Actions Release workflow logs when tags are pushed.
- Do not claim Windows launch success unless Windows 11 user QA or a real
  Windows execution environment confirms it.
- Do not publish macOS GitHub Release artifacts without Developer ID signing and
  Apple notarization.
- Local macOS internal QA may use `npm run tauri:build` output, but that result
  must be reported as `internal local QA only`, not team-distribution verified.
- Do not claim `tauri:dev` success from Vite logs alone. Native window or native
  process evidence is required.

For remote-related issues:
- If no real OE-Linux target is available locally, use captured fixture output
  for parser tests and state live target verification as unverified.

## Handoff Expectation

At every Sprint 06 handoff, report:
- which issue was worked
- what changed
- what internal checks passed
- what remains unverified
- which issue needs user QA next
