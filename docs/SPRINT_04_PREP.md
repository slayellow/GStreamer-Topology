# Sprint 04 Preparation

Date: 2026-04-26

Branch:
- `sprint_04`

Base:
- `main` after Sprint 03 merge commit `aa75c23`

## Sprint Goal

Stabilize the topology verification workflow for real user files by making long
`.pld` source highlighting reliable, then simplify the Local/Remote entry
experience so Remote Server access is clearly separated from the local pipeline
workflow.

## Sprint Board Items

Primary implementation order:

1. `#16` `스프린트 04: 파일 가져오기 미리보기와 토폴로지 생성 단계 분리`
2. `#17` `스프린트 04: 아이콘 중심 캔버스 툴바와 보조 패널 Drawer 전환`
3. `#13` `스프린트 04: Local/Remote 진입 UX와 Remote Server 접속 모달 재설계`
4. `#14` `스프린트 04: 긴 Pipeline 원문 하이라이트 안정화`

Rationale:
- `#16` comes first because file import no longer goes directly to canvas; the
  new preview/edit step defines how users enter the workspace.
- `#17` comes second because the workspace is now canvas-first with icon-driven
  drawers for inspector, source text, and parser diagnostics.
- `#13` follows the new shell structure with Remote Server status badges and a
  modal-based remote connection flow.
- `#14` remains in the sprint, but source highlighting should be finalized on
  top of the new source drawer interaction.

GitHub Project note:
- Parent board: `GStreamer Topology Sprint Board`
- Sprint execution board: `Sprint 04`
- Project URL: `https://github.com/users/slayellow/projects/2`
- The `Sprint 04` Project is linked to the `GStreamer-Topology` repository.
- Issues `#13` and `#14` are added to both the parent board and the `Sprint 04`
  Project.
- Issues `#16` and `#17` were added after the Sprint 04 UI/UX requirements were
  refined.
- Current `Sprint 04` Project state:
  - `#16` is `In Progress`
  - `#17` is `In Progress`
  - `#13` is `In Progress`
  - `#14` is `Todo`

## Expert Loop

Use the stable aliases defined in `AGENTS.md` and `docs/PROCESS_POLICY.md`.

- `Atlas`: planner
- `Forge`: developer
- `Loom`: designer
- `Beacon`: QA

### Atlas Planning Summary

Goal:
- Make the user able to trust graph-to-source navigation on long `.pld` files, then
  clarify the Local/Remote entry flow.

Smallest useful slices:
- Add a file import preview/edit screen before workspace navigation.
- Move workspace support panels into icon-triggered drawers so the canvas is
  maximized by default.
- Surface GStreamer API and Remote Server connection status through status
  badges.
- Add regression coverage that validates source spans for
  `fixtures/pipelines/26_release_record_smoothing.pld` and
  `fixtures/pipelines/27_pipmux.pld`.
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

Likely files for `#16` and `#17`:
- `src/app/AppShell.tsx`
- `src/app/status.ts`
- `src/components/Icon.tsx`
- `src/components/IconButton.tsx`
- `src/components/ConnectionBadge.tsx`
- `src/features/import-preview/ImportPreviewScreen.tsx`
- `src/features/home/HomeScreen.tsx`
- `src/features/workspace/WorkspaceShell.tsx`
- `src/styles/app-shell.css`

Likely files for `#13`:
- `src/features/home/HomeScreen.tsx`
- `src/app/AppShell.tsx`
- `src/app/backend.ts`
- `src/styles/app-shell.css`

Implementation order:
- Start with the import preview/edit screen and reuse parsed preview text for
  final topology generation.
- Then switch workspace panels to icon-triggered drawers and keep the canvas
  full-width by default.
- Add GStreamer API and Remote Server status badges across home, preview, and
  workspace.
- Finish long `.pld` source highlight reliability after the source panel lives in
  the drawer.

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

For `#16` and `#17`:
- Show a `파일 미리보기` step before opening the workspace.
- Keep workspace support tools as icon actions in the top-right area.
- Default the workspace to canvas-first, with drawers opening only on demand.
- Ensure icon-only controls still have labels, tooltips, and keyboard focus
  feedback.

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
- Open `fixtures/pipelines/26_release_record_smoothing.pld`.
- Open `Pipeline 원문`.
- Click several elements and confirm the source highlight is visible and moves.
- Repeat with `fixtures/pipelines/27_pipmux.pld`.
- Record fixture name, clicked element, and panel state for any missing
  highlight.

User-visible checks for `#13`:
- Confirm the first screen clearly separates Local and Remote Server.
- Open the Remote Server modal.
- Enter invalid connection data and confirm the failure is understandable.
- Confirm the Local file and paste flows still work after failed remote access.

User-visible checks for `#16` and `#17`:
- Select a local file and confirm the app opens `파일 미리보기`, not the
  workspace.
- Edit preview text and confirm `토폴로지 생성` uses the edited text.
- Confirm the workspace opens with inspector/source/diagnostics hidden by
  default.
- Use the top-right icons to open and close inspector, source text, and parser
  diagnostics.
- Confirm icon controls show focus outlines and useful labels/tooltips.

Unverified until a real target is available:
- `OE-Linux` SSH authentication
- SFTP remote read/listing
- remote `gst-inspect-1.0` metadata parsing
- Windows/Linux packaging and native WebView behavior

## Acceptance Criteria

For `#14`:
- `fixtures/pipelines/26_release_record_smoothing.pld` and
  `fixtures/pipelines/27_pipmux.pld` parse without
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

For `#16` and `#17`:
- File selection opens a preview/edit screen before workspace navigation.
- The preview screen displays normalized pipeline text for local `.pld` files.
- Workspace opens with the topology canvas as the dominant view.
- Inspector, source text, and parser diagnostics are hidden by default and can
  be toggled from top-right icons.
- Icon-only controls provide accessible labels, title tooltips, and focus
  states.

## Handoff Rule For Sprint 04

Before handing work to the user for QA:
- Update the relevant GitHub issue body with current implementation status.
- Post `Beacon` expert QA results as a Korean comment on the issue.
- Move the issue to `In Progress` while implementation or user QA is active.
- Move the issue to `Done` only after user QA passes.
- Record any user-reported failure as same-sprint rework or next-sprint backlog.
