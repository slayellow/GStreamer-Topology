# Agent Handoff Guide

This file is the fast-start entry point for future coding agents.

Read this file and `docs/PROCESS_POLICY.md` before starting feature work.

## Current Status

This repository is in active prototype mode, not planning-only mode.

What exists:
- `Tauri 2` desktop scaffold
- `React + TypeScript` app shell with home and workspace flows
- Rust backend with RTF normalization and tolerant pipeline parsing
- `React Flow + ELK` topology rendering
- Basic inspector and diagnostics surface
- Local file import and pasted-text parsing
- Remote SSH probe and remote pipeline load commands
- Parser tests for the sample fixtures
- Working `lint`, `build`, `cargo test`, and `tauri:dev` paths

What is still incomplete or partially verified:
- Real `OE-Linux` target verification for SSH/SFTP and metadata enrichment
- Remote file browser UX
- Full remote element property enrichment
- Export, search hardening, and large-graph UX improvements
- Packaging and signing for `Windows`, `Linux`, and `macOS`

## Source Of Truth

Read these documents in order before making implementation decisions:

1. `docs/PROCESS_POLICY.md`
2. `docs/PRD.md`
3. `docs/ARCHITECTURE.md`
4. `docs/IMPLEMENTATION_PLAN.md`
5. `docs/REPOSITORY_STRUCTURE.md`
6. `docs/TECH_SPIKES.md`

If there is a conflict:
- User instruction wins
- `docs/PROCESS_POLICY.md` governs execution, verification, and handoff
- The remaining docs govern scope, architecture, and delivery order

## Locked Decisions

These decisions are already made unless the user explicitly changes them:

- The local desktop app must run on `Windows`, `Linux`, and `macOS`
- Remote targets are `OE-Linux`
- Primary stack: `Tauri 2 + Rust + React + TypeScript`
- Graph rendering: `React Flow + ELK layered`
- Remote access: `SSH + SFTP`, read-only in MVP
- Parsing must work without bundling local GStreamer
- Remote targets are the authority for plugin and element metadata
- Initial credential storage must use OS secure storage, not plain files
- Design language: `Miro`-inspired "Technical Canvas"
- MVP supports one target connection at a time, not fleet management

## Process Policy

Keep these rules in mind even if you only skim this file:

- Work in thin slices with one coherent user-visible outcome.
- For every substantial feature or workflow change, run the expert subagent loop:
  `planner -> developer -> designer -> QA`.
- Use stable user-facing subagent aliases during sprint work:
  `Atlas` for planning, `Forge` for development, `Loom` for design, and
  `Beacon` for QA. The coordinator routes user requests to the matching role
  even if the underlying subagent session ID changes.
- Build and verify the current slice end to end before starting adjacent work.
- Prefer partial success plus diagnostics over hard failure.
- Preserve source-span mapping whenever parsing, graph IR, selection, or
  diagnostics are touched.
- If verification is incomplete, say so explicitly. Do not imply success.

Detailed checklists and handoff rules live in `docs/PROCESS_POLICY.md`.

## Lessons Learned

Do not repeat the `tauri:dev` verification mistake from `2026-04-22`.

- Do not treat `npm run tauri:dev` as successful just because Vite started,
  Rust compiled, the command kept running, or logs looked steady.
- `tauri:dev` is verified only when the native Tauri app window launches, or
  the spawned native app process is confirmed alive without immediately
  crashing.
- Record exact verification evidence in the handoff.
- If full launch cannot be confirmed, report `launch unverified`, not
  `working`.

## Primary Samples

Use these as canonical fixtures during parser and workspace development:

- `26_release_record_smoothing.pld.rtf`
- `27_pipmux.pld.rtf`

These files contain:
- `tee`
- `funnel`
- `output-selector`
- request-pad links such as `mixer.sink_1`
- named references such as `eoraw.` and `eismux.`
- caps strings with memory features
- placeholders such as `{EIS_RANGE}`

## Current MVP Baseline

The current prototype can:

- Open a local sample file and normalize RTF-wrapped pipeline text
- Parse pipeline text into a tolerant graph model
- Render a topology workspace with selectable nodes
- Show basic inspector details and parse diagnostics
- Accept remote connection inputs and call backend SSH probe/load commands

The current prototype does not yet prove:

- Real target metadata enrichment on a live `OE-Linux` device
- Full remote browsing workflow
- Cross-platform packaging quality

## Preferred Next Slice

Unless the user redirects, the next meaningful implementation order is:

1. Verify SSH/SFTP behavior against a real `OE-Linux` target
2. Build the remote file browser and open flow
3. Enrich inspector details from remote `gst-inspect` output
4. Add search, export, and large-graph usability improvements

## Handoff Expectation

At the end of a task, future agents should report:

- what was completed
- what was verified
- what remains unverified
- the next smallest recommended step
