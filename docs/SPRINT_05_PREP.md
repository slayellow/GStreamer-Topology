# Sprint 05 Preparation

Date: 2026-04-26

Branch:
- `sprint_05`

Base:
- `main` after Sprint 04 merge commit `0a2c0ae`

## Sprint Setup Status

GitHub Project note:
- Parent board: `GStreamer Topology Sprint Board`
- Sprint execution board: `Sprint 05`
- Project URL: `https://github.com/users/slayellow/projects/3`
- The `Sprint 05` Project is linked to the `GStreamer-Topology` repository.
- Status field uses `Todo`, `In Progress`, and `Done`.
- Sprint label: `sprint-05`

Sprint board items:
- `#19` `스프린트 05 후보: 캔버스 원문 하이라이트 잔여 안정화`
- `#21` `스프린트 05: 워크스페이스 상단 정보 구조와 캔버스 카피 정리`
- `#22` `스프린트 05: 노드 포트와 Edge Label 가독성 개선`
- `#23` `스프린트 05: Inspector Source Diagnostics 하단 Resizable 패널 전환`
- `#24` `스프린트 05: 파서 진단 목적 재정의와 Actionable Diagnostics UX`
- `#25` `스프린트 05: 토폴로지 Export PNG JPG SVG 저장 기능`
- Parent board status: `Todo`
- Sprint 05 Project status: `Todo`

## Expected Sprint Theme

The user has indicated Sprint 05 will likely include canvas UI changes.

Known carry-over from Sprint 04:
- Long RTF source highlighting is mostly usable, but intermittent misses still
  exist in the canvas/source interaction.
- The residual issue is accepted for Sprint 04 and tracked as `#19`.
- Treat source-highlight reliability as part of the upcoming canvas UX work
  unless the user redirects.

## Sprint 05 Requirement Intake

User requirements captured on 2026-04-26:
- Improve the Workspace/Canvas UI after topology generation.
- Remove noisy Workspace heading/subtitle/chips.
- Keep `GStreamer API` visible and show whether the current authority is
  `Local` or `Remote`.
- Use English labels inside the canvas surface, such as `Topology Canvas`,
  `Nodes`, `Edges`, and layout actions.
- Improve long edge label readability, especially caps strings such as
  `video/x-raw(...)`.
- Simplify node-side `SINK`/`SRC` labels so they do not cover element names.
- Remove default `SRC -> SINK` edge text and show only meaningful edge details.
- Move `Pipeline 원문` and `Parser Diagnostics` to the bottom area.
- Include `Inspector`, `Source`, and `Diagnostics` in resizable bottom panels.
- Clarify what `Parser Diagnostics` means and avoid implying runtime pipeline
  execution validation.
- Remove or redesign the inspector icon red dot.
- Add export actions for `PNG`, `JPG`, and `SVG` with user-selected save path.

## Recommended Implementation Order

1. `#21` Workspace topbar/canvas copy cleanup.
   - This is the smallest visible cleanup and reduces noise before deeper
     layout work.
2. `#22` Node port and edge label readability.
   - This directly addresses graph readability and should come before export.
3. `#23` Bottom resizable panels.
   - This changes the workspace layout and must preserve source highlight
     behavior from `#19`.
4. `#19` Residual source highlight stabilization.
   - If `#23` changes panel scroll behavior, verify and fix source highlight in
     the new bottom-panel structure.
5. `#24` Parser diagnostics purpose and actionable UX.
   - This can build on the new bottom panel and should not be confused with
     runtime GStreamer validation.
6. `#25` Export `PNG`/`JPG`/`SVG`.
   - Implement after the canvas visual structure is stable. If full raster
     export is blocked by Tauri/canvas serialization, ship `SVG` first and
     split raster export into a follow-up.

## Expert Intake Summary

Atlas:
- Treat Sprint 05 as canvas/workspace stabilization.
- Keep `#19` as a priority because it protects source-span trust.
- Split export into its own issue because it touches Tauri save dialogs and
  image serialization.

Forge:
- Primary code surfaces are `WorkspaceShell`, `GraphCanvas`, `toReactFlow`,
  `fromBackend`, `TechnicalNode`, `SourceTextPanel`, `DiagnosticsPanel`,
  `IconButton`, `Icon`, and workspace styles.
- Export may require Tauri dialog/filesystem capability changes.
- Edge label length is risky because the ELK layout may not account for label
  width.

Loom:
- Shift the screen from UI explanation toward a canvas-first workspace.
- Keep Technical Canvas styling but reduce chips and long copy.
- Prefer a bottom tabbed panel for `Inspector`, `Source`, and `Diagnostics`.
- Show only meaningful edge labels by default; emphasize connected edges when
  a node is selected.

Beacon:
- Use `26_release_record_smoothing.pld.rtf`, `27_pipmux.pld.rtf`, and short
  fixtures `01` to `04` for QA.
- Export must be verified with actual saved files, not only UI clicks.
- `tauri:dev` still requires native window/process evidence.

## Sprint Intake Rule

When the user provides Sprint 05 requirements:
- Read `AGENTS.md` and `docs/PROCESS_POLICY.md` first.
- Consult the Karpathy Guidelines before substantial implementation.
- Use the expert loop with the stable aliases:
  - `Atlas`: planner
  - `Forge`: developer
  - `Loom`: designer
  - `Beacon`: QA
- Create one Korean GitHub issue per independently testable feature or bug.
- Add each Sprint 05 issue to both:
  - `GStreamer Topology Sprint Board`
  - `Sprint 05`
- Add labels:
  - `sprint`
  - `sprint-05`
  - the relevant priority/type labels
  - `role:planner`, `role:developer`, `role:designer`, `role:qa`
- Keep issues in `Todo` until implementation starts, move to `In Progress`
  during implementation/user QA, and move to `Done` only after user QA passes.

## Expert Loop Reminder

Atlas should:
- Restate the user-visible goal.
- Break Sprint 05 requirements into the smallest shippable slices.
- Identify whether `#19` should be merged into a broader canvas UX issue or
  kept as a separate bug.
- Define acceptance criteria and stop conditions.

Forge should:
- Implement only the active slice.
- Preserve graph/source-span mapping whenever canvas, selection, source text,
  or diagnostics are touched.
- Avoid broad graph architecture changes unless the issue explicitly requires
  them.

Loom should:
- Keep the Technical Canvas design direction.
- Prioritize graph readability, selection feedback, and discoverable controls.
- Ensure icon-only actions keep accessible labels and visible focus states.

Beacon should:
- Verify each slice against the issue acceptance criteria.
- Include at least one happy path and one tolerant-failure path when relevant.
- Record unverified areas explicitly.
- Confirm `npm run tauri:dev` only with a native window or native process
  evidence.

## Standard Verification

Before handing Sprint 05 work to the user for QA, run:
- `git diff --check`
- `npm run lint`
- `npm run build`
- `cd src-tauri && cargo test`
- `npm run tauri:dev`

`tauri:dev` is verified only when:
- the native app window launches, or
- the native app process is confirmed alive without immediately crashing.

## Handoff Expectation

At every Sprint 05 handoff, report:
- what was completed
- what was verified
- what remains unverified
- which GitHub issue needs user QA next
