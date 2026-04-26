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

Initial board item:
- `#19` `스프린트 05 후보: 캔버스 원문 하이라이트 잔여 안정화`
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

