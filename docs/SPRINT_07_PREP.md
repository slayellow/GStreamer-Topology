# Sprint 07 Preparation

Date: 2026-05-05

Branch:
- `sprint_07`

Base:
- `main` after Sprint 06 merge commit `79f7e55`

## Sprint Setup Status

GitHub Project note:
- Parent board: `GStreamer Topology Sprint Board`
- Sprint execution board: `Sprint 07`
- Project URL: `https://github.com/users/slayellow/projects/5`
- The `Sprint 07` Project is linked to the `GStreamer-Topology` repository.
- Status field uses `Todo`, `In Progress`, and `Done`.
- Sprint label: `sprint-07`

Sprint board items:
- `#25` `스프린트 07: 토폴로지 Export PNG/JPG 전체 저장 기능`
- `#33` `스프린트 07: Windows 11 실행 시 명령프롬프트 창 제거`

Setup completed:
- `#25` and `#33` are on the parent board.
- `#25` and `#33` are on the `Sprint 07` Project.
- `#25` and `#33` have the `sprint-07` label.
- Sprint-specific planning comments were added to both issues in Korean.
- Both issues are currently in `Todo`.

## Expected Sprint Theme

Sprint 07 should focus on post-release feedback and one carry-over usability
gap from Sprint 06.

The sprint should answer two practical questions:
- Can exported topology images include the full graph, not only the visible
  viewport?
- Can the Windows installer launch like a normal GUI desktop app without a
  console window?

## Issue Triage

### `#25` Full topology PNG/JPG export

Priority:
- `P1`, carry-over from Sprint 06

User evidence:
- Sprint 06 export saved only the currently visible canvas viewport.
- The expected behavior is exporting the entire topology graph bounds,
  including off-screen nodes, edges, edge labels, and caps labels.

Sprint 07 outcome:
- Export defaults to full topology graph bounds.
- PNG and JPG files are non-empty and include off-screen graph content.
- Canvas controls, minimap, status badges, and other UI chrome are excluded
  from the exported image.
- `fixtures/pipelines/26_release_record_smoothing.pld` and
  `fixtures/pipelines/27_pipmux.pld` are used as large-graph verification
  samples.

### `#33` Windows console window removal

Priority:
- `P2`

User evidence:
- On Windows 11, launching the installed MSI build opens both the GUI and a
  command prompt window.
- Closing the command prompt also terminates the app.

Sprint 07 outcome:
- Windows Release builds run as GUI apps without a console window.
- Start Menu shortcut and direct exe launch both open only the GUI.
- A fallback way to collect diagnostics is documented because the console will
  no longer be visible.

## Recommended Implementation Order

1. `#33` Windows console window removal.
   - This is likely a small packaging/build setting change and should reduce
     friction before the next Windows team QA pass.
2. `#25` full topology export.
   - This is the larger functional slice and should be fixture-driven with
     `01`, `26`, and `27` pipeline files.

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

Before handing Sprint 07 work to the user for QA, run the relevant subset of:
- `git diff --check`
- `npm run lint`
- `npm run build`
- `cd src-tauri && cargo test`
- `npm run tauri:build`

For export-related issues:
- Verify small and large fixtures.
- Confirm saved files are non-empty.
- Confirm full graph bounds are included, not only the current viewport.
- State whether visual quality was internally verified or left to user QA.

For Windows packaging issues:
- Verify source configuration and build output as far as possible locally.
- Use GitHub Actions Release workflow logs when artifacts are generated.
- Do not claim Windows 11 launch success unless Windows user QA or a real
  Windows execution environment confirms it.

## Handoff Expectation

At every Sprint 07 handoff, report:
- which issue was worked
- what changed
- what internal checks passed
- what remains unverified
- which issue needs user QA next
