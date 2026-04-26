# Sprint 04 Preparation

Date: 2026-04-26

Branch:
- `sprint_04`

Base:
- `main` after Sprint 03 merge commit `aa75c23`

## Sprint Goal

Stabilize the topology verification workflow for real user files by making long
RTF source highlighting reliable, then simplify the Local/Remote entry
experience so Remote Server access is clearly separated from the local pipeline
workflow.

## Sprint Board Items

Primary implementation order:

1. `#14` `스프린트 04: 긴 RTF Pipeline 원문 하이라이트 안정화`
2. `#13` `스프린트 04: Local/Remote 진입 UX와 Remote Server 접속 모달 재설계`

Rationale:
- `#14` comes first because source-span highlighting is the trust layer between
  the rendered topology and the original pipeline text.
- `#13` comes second because the Remote Server UX is broader and should not
  distract from fixing the user-reported Sprint 03 regression.

GitHub Project note:
- GitHub CLI and the currently available GraphQL mutations can update issues,
  labels, comments, and project item status.
- They do not expose a safe project view create/rename mutation in this
  environment.
- If a dedicated Project View is needed, create or rename it in the GitHub web
  UI as `Sprint 04`.

## Expert Loop

Use the stable aliases defined in `AGENTS.md` and `docs/PROCESS_POLICY.md`.

- `Atlas`: planner
- `Forge`: developer
- `Loom`: designer
- `Beacon`: QA

### Atlas Planning Summary

Goal:
- Make the user able to trust graph-to-source navigation on long RTF files, then
  clarify the Local/Remote entry flow.

Smallest useful slices:
- Add regression coverage that validates source spans for
  `26_release_record_smoothing.pld.rtf` and `27_pipmux.pld.rtf`.
- Fix source-panel highlighting and scrolling for long normalized text.
- Redesign the home entry so Local remains the default and Remote Server opens
  from a separate modal.

Stop conditions:
- Stop if source-span mapping cannot be preserved with fixture evidence.
- Stop if Remote Server work requires write access to the target or plaintext
  credential persistence.
- Mark real remote behavior as unverified unless tested against a real
  `OE-Linux` target or a controlled test target.

### Forge Implementation Summary

Likely files for `#14`:
- `src/features/workspace/SourceTextPanel.tsx`
- `src/features/workspace/WorkspaceShell.tsx`
- `src/graph/fromBackend.ts`
- `src-tauri/src/parser/pipeline.rs`

Likely files for `#13`:
- `src/features/home/HomeScreen.tsx`
- `src/app/AppShell.tsx`
- `src/app/backend.ts`
- `src/styles/app-shell.css`

Implementation order:
- Start with backend parser/source-span regression tests for long RTF fixtures.
- Then fix frontend span validation, highlight rendering, and source-panel
  scroll/focus behavior.
- After `#14` passes, split Remote Server UX into a modal-oriented UI slice.

Risks:
- Rust spans are byte offsets, while JavaScript string slicing uses UTF-16 code
  units. Conversion must be handled deliberately if non-ASCII text is involved.
- Nested scroll containers can make `scrollIntoView` appear unreliable unless
  the highlighted target and panel container are controlled.
- Actual Remote Server success cannot be fully verified without an `OE-Linux`
  target.

### Loom Design Summary

Design direction:
- Keep the Technical Canvas direction: calm off-white workspace, clear blue
  selection state, compact desktop chrome, low visual noise.

For `#14`:
- When an element is selected, the `Pipeline 원문` panel should visibly move to
  the selected span.
- If the source position cannot be found, show an explicit recoverable warning
  instead of silently failing.
- Show the currently selected element near the source panel so users can tell
  what the highlight represents.

For `#13`:
- Keep Local as the default path.
- Move Remote Server inputs out of the main home layout into a focused modal.
- Use copy that makes the Remote Server role clear: read GStreamer metadata
  from the target device, not general fleet management.

### Beacon QA Summary

Required automated checks:
- `npm run lint`
- `npm run build`
- `cd src-tauri && cargo test`
- `npm run tauri:dev`

`tauri:dev` rule:
- It only counts as verified when the native Tauri window launches, or the
  native app process is confirmed alive without immediately crashing.

User-visible checks for `#14`:
- Open `26_release_record_smoothing.pld.rtf`.
- Open `Pipeline 원문`.
- Click several elements and confirm the source highlight is visible and moves.
- Repeat with `27_pipmux.pld.rtf`.
- Record fixture name, clicked element, and panel state for any missing
  highlight.

User-visible checks for `#13`:
- Confirm the first screen clearly separates Local and Remote Server.
- Open the Remote Server modal.
- Enter invalid connection data and confirm the failure is understandable.
- Confirm the Local file and paste flows still work after failed remote access.

Unverified until a real target is available:
- `OE-Linux` SSH authentication
- SFTP remote read/listing
- remote `gst-inspect-1.0` metadata parsing
- Windows/Linux packaging and native WebView behavior

## Acceptance Criteria

For `#14`:
- `26_release_record_smoothing.pld.rtf` and `27_pipmux.pld.rtf` parse without
  source-span bounds regressions.
- Selected graph nodes with source spans highlight the corresponding normalized
  source text.
- Long source text scrolls or focuses to the selected highlight.
- Highlight failure is shown as a recoverable diagnostic, not a silent miss.
- Existing short fixtures `01` to `04` keep their source-view behavior.

For `#13`:
- Local file open and pasted-text topology generation remain the primary first
  screen actions.
- Remote Server connection opens in a modal/dialog instead of occupying the
  default home layout.
- Invalid remote connection attempts show a clear failure state.
- Failed remote attempts do not destroy or block the Local workflow.
- UI copy remains Korean except code identifiers, commands, file paths, APIs,
  and GStreamer syntax.

## Handoff Rule For Sprint 04

Before handing work to the user for QA:
- Update the relevant GitHub issue body with current implementation status.
- Post `Beacon` expert QA results as a Korean comment on the issue.
- Move the issue to `In Progress` while implementation or user QA is active.
- Move the issue to `Done` only after user QA passes.
- Record any user-reported failure as same-sprint rework or next-sprint backlog.
