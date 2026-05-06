# Sprint 08 Preparation

Date: 2026-05-06

Branch:
- `sprint_08`

Base:
- `main` after Sprint 07 merge commit `0827656`

## Sprint Setup Status

GitHub Project note:
- Parent board: `GStreamer Topology Sprint Board`
- Sprint execution board: `Sprint 08`
- Project URL: `https://github.com/users/slayellow/projects/6`
- The `Sprint 08` Project is linked to the `GStreamer-Topology` repository.
- Status field uses `Todo`, `In Progress`, and `Done`.
- Sprint label: `sprint-08`

Sprint board items:
- `#37` `스프린트 08: Edge Crossing 최소화 라우팅 개선`
- `#38` `스프린트 08: Export 이미지 그림자/ghosting 제거`
- `#40` `스프린트 08: Windows 11 소형/대형 Pipeline Canvas 반응성 개선`
- `#41` `스프린트 08: Pipeline Simulation 실행 검증 기능`

Setup completed:
- `#37`, `#38`, `#40`, and `#41` are on the parent board.
- `#37`, `#38`, `#40`, and `#41` are on the `Sprint 08` Project.
- `#37`, `#38`, `#40`, and `#41` have the `sprint-08` label.
- `#37`, `#38`, `#40`, and `#41` are currently `Todo`.
- `#40` was expanded with role ownership, acceptance criteria, and a detailed
  user QA checklist after Windows 11 field testing.
- `#41` was created from the Simulation requirement and includes Local/Remote
  GStreamer API availability gating plus execution-diagnostic UX.

## Expected Sprint Theme

Sprint 08 should focus on Windows field-test feedback and visual output
hardening.

The sprint should answer these practical questions:
- Can large topology graphs remain responsive on Windows 11 Release builds?
- Can small topology graphs remain responsive on Windows 11 Release builds?
- Can Edge Crossing be reduced without regressing the Sprint 07 endpoint and
  highlight improvements?
- Can PNG/JPG export produce clean documentation-ready diagrams without
  shadow or ghosting artifacts?
- Can users run a bounded Local/Remote GStreamer simulation check from the
  topology screen and see clear success/failure diagnostics?

## Issue Triage

### `#40` Windows 11 small/large Pipeline Canvas responsiveness

Priority:
- `P1`

User evidence:
- On Windows 11, after opening a PLD file and generating topology, clicking a
  Canvas Element can take roughly 3 to 4 seconds to respond.
- The issue reproduces on both small sample pipelines, such as the first PLD
  example, and large `26`/`27` pipelines.
- Clicking an Element in the Pipeline source panel and moving to the Canvas is
  also slow.
- This is difficult to capture with a screenshot, but it directly affects the
  next team distribution goal.

Sprint 08 target outcome:
- Reduce unnecessary full graph re-rendering during selection and source-text
  navigation.
- Keep Element selection, connected Edge highlight, Inspector updates, and
  source-to-canvas movement functional.
- Prefer measurable and surgical optimizations before considering large
  renderer changes.

### `#41` Pipeline Simulation execution validation

Priority:
- `P1`

User evidence:
- Topology visualization can parse and render a Pipeline, but it does not
  verify whether the Pipeline can run in the actual Local or Remote GStreamer
  environment.
- The user wants a Simulation icon on the topology screen that uses the
  available Local/Remote GStreamer API to execute a bounded validation and show
  syntax/runtime errors.
- If the Local/Remote GStreamer API is unavailable, the app should block
  Simulation before execution and explain why.

Sprint 08 target outcome:
- Add a topology toolbar Simulation action.
- Gate Simulation by Local/Remote GStreamer API availability.
- Run a bounded Local/Remote `gst-launch-1.0` validation path where available.
- Show success/failure diagnostics without blocking topology visualization.

### `#37` Edge Crossing minimization

Priority:
- `P2`, carry-over from Sprint 07

User evidence:
- Sprint 07 fixed Edge lines covering Element cards and improved SRC/SINK
  endpoint distribution.
- Some Edge Crossing remains in branch/merge-heavy areas.

Sprint 08 target outcome:
- Improve ELK/React Flow routing configuration and port ordering where it can
  reduce crossing safely.
- Preserve the Sprint 07 behavior that keeps edges away from Element text and
  distributes endpoints across node rails.

### `#38` Export image shadow/ghosting removal

Priority:
- `P2`, carry-over from Sprint 07

User evidence:
- Sprint 07 full topology PNG/JPG export works, but exported images can show
  shadow or ghosting artifacts in front of Element cards.

Sprint 08 target outcome:
- Export PNG/JPG as clean technical diagrams.
- Keep full graph export bounds and exclude app UI chrome.
- Ensure export-only styling does not leak back into the live Canvas.

## Recommended Implementation Order

1. `#40` Windows 11 small/large Pipeline Canvas responsiveness.
   - This is the highest-risk field-test issue and should be tackled before
     adding more visual complexity.
2. `#41` Pipeline Simulation execution validation.
   - This is a new user-visible capability and should be kept as a bounded MVP:
     availability gating, short timeout, and clear diagnostics.
3. `#37` Edge Crossing minimization.
   - This may touch the same graph mapping/layout path as `#40`, so it should
     follow performance work to avoid optimizing code that will change again.
4. `#38` Export image shadow/ghosting removal.
   - This is important for documentation quality but less blocking than
     interactive responsiveness.

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
- Treat Windows Release responsiveness as a first-class verification target.

Loom:
- Check canvas readability, selection feedback, and export visual quality.
- Preserve the Miro-inspired Technical Canvas direction.
- Avoid visual changes that increase large-graph render cost.

Beacon:
- Verify the slice against issue criteria.
- Record internal QA results as a Korean comment on the matching GitHub issue.
- Explicitly state what remains unverified for user QA.

## Verification Standard

Before handing Sprint 08 work to the user for QA, run the relevant subset of:
- `git diff --check`
- `npm run lint`
- `npm run build`
- `cd src-tauri && cargo test`
- `npm run tauri:build`

For performance-related work:
- Use `fixtures/pipelines/01_videotestsrc_linear.pld` or the first sample PLD.
- Use `fixtures/pipelines/26_release_record_smoothing.pld`.
- Use `fixtures/pipelines/27_pipmux.pld`.
- Verify Element click, source-text click, pan, zoom, and Inspector update.
- If Windows is not directly available to the agent, report Windows launch or
  responsiveness as user-QA-required instead of claiming it verified.

For simulation work:
- Verify the no-GStreamer-API path when local `gst-launch-1.0` is unavailable.
- Verify a successful local simulation path when local `gst-launch-1.0` is
  available.
- Verify an invalid Pipeline returns a clear failure diagnostic.
- Verify Remote simulation is blocked with a clear message when no remote
  target is connected or remote API availability is unknown.

For edge-routing work:
- Verify `02_videotestsrc_tee_branch.pld`.
- Verify the large `26` and `27` fixtures.
- Confirm edges do not regress to covering Element labels.

For export work:
- Verify PNG and JPG with a small fixture and at least one large fixture.
- Confirm controls, minimap, status badges, and app chrome are excluded.
- Confirm full graph bounds are exported, not only the current viewport.

Sprint closeout must also trigger the `Desktop Release` workflow and confirm
Windows/Linux installer assets plus checksum files before the sprint is called
complete.

## User QA Triage On 2026-05-06

User QA result:
- `#37` Edge Crossing minimization: `SUCCESS`.
  - The user confirmed the routing is better than before and accepted the
    remaining crossing as good enough for now.
  - If another team member finds the remaining crossing uncomfortable during
    real use, create a new follow-up issue in a later sprint.
- `#38` Export ghosting removal: `SUCCESS`.
  - The user confirmed the shadow/ghosting artifact is gone.
- `#40` Windows 11 small/large Canvas responsiveness: accepted and closed.
  - The user chose to close the issue and register a new issue only if Windows
    use exposes a remaining problem.
- `#41` Pipeline Simulation: `SUCCESS` on macOS Local.
  - The user confirmed the Mac flow.
  - Real OE-Linux Remote validation remains field-verification-required.
  - The user will run the Remote/camera-device path later and create an issue
    if a problem appears.

Same-sprint rework:
- None identified from the user QA comments.

Known unverified field checks:
- Remote Simulation on an actual OE-Linux target with camera hardware.
- Simulation behavior for production-scale `26` Pipeline on the target device,
  especially if the Pipeline is expected to run longer than the bounded
  five-second MVP validation window.

## Parser Diagnostics Scope

`Parser Diagnostics` is a static parser recovery report, not a GStreamer runtime
simulation result.

It exists to explain how the tolerant parser interpreted pipeline text while
still rendering a topology. Typical diagnostics include:
- text normalization notes
- missing `!` link operators between adjacent tokens
- loose or ignored element tokens
- duplicate element instance names
- unresolved named references such as `foo.`
- dangling caps strings that could not be connected to a segment

It does not prove:
- that `gst-launch-1.0` can run the Pipeline
- that every element exists in the Local/Remote GStreamer registry
- that properties are valid for a specific plugin version
- that request pads negotiate successfully at runtime
- that camera/display/device resources are available

Use `Parser Diagnostics` to debug topology parsing and source-span mapping.
Use `Simulation` to check bounded Local/Remote GStreamer execution failures.
Use `Inspector` to inspect Local/Remote element metadata when the target
GStreamer API is available.

## Handoff Expectation

At every Sprint 08 handoff, report:
- which issue was worked
- what changed
- what internal checks passed
- what remains unverified
- which issue needs user QA next
