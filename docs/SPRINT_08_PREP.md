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
- `#40` `스프린트 08: Windows 11 대형 Pipeline Canvas 반응성 개선`

Setup completed:
- `#37`, `#38`, and `#40` are on the parent board.
- `#37`, `#38`, and `#40` are on the `Sprint 08` Project.
- `#37`, `#38`, and `#40` have the `sprint-08` label.
- `#37`, `#38`, and `#40` are currently `Todo`.
- `#40` was expanded with role ownership, acceptance criteria, and a detailed
  user QA checklist after Windows 11 field testing.

## Expected Sprint Theme

Sprint 08 should focus on Windows field-test feedback and visual output
hardening.

The sprint should answer these practical questions:
- Can large topology graphs remain responsive on Windows 11 Release builds?
- Can Edge Crossing be reduced without regressing the Sprint 07 endpoint and
  highlight improvements?
- Can PNG/JPG export produce clean documentation-ready diagrams without
  shadow or ghosting artifacts?

## Issue Triage

### `#40` Windows 11 large-pipeline Canvas responsiveness

Priority:
- `P1`

User evidence:
- On Windows 11, after opening a PLD file and generating topology, clicking a
  Canvas Element can take roughly 3 to 4 seconds to respond.
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

1. `#40` Windows 11 large-pipeline Canvas responsiveness.
   - This is the highest-risk field-test issue and should be tackled before
     adding more visual complexity.
2. `#37` Edge Crossing minimization.
   - This may touch the same graph mapping/layout path as `#40`, so it should
     follow performance work to avoid optimizing code that will change again.
3. `#38` Export image shadow/ghosting removal.
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
- Use `fixtures/pipelines/26_release_record_smoothing.pld`.
- Use `fixtures/pipelines/27_pipmux.pld`.
- Verify Element click, source-text click, pan, zoom, and Inspector update.
- If Windows is not directly available to the agent, report Windows launch or
  responsiveness as user-QA-required instead of claiming it verified.

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

## Handoff Expectation

At every Sprint 08 handoff, report:
- which issue was worked
- what changed
- what internal checks passed
- what remains unverified
- which issue needs user QA next
