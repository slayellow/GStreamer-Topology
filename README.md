# GStreamer To Topology

Cross-platform desktop app for loading GStreamer pipeline text, visualizing the
topology, and enriching the view with metadata from remote `OE-Linux` targets.

## Current Repository State

This repository is no longer planning-only.

What exists today:
- `Tauri 2` desktop scaffold
- `React + TypeScript` frontend
- Rust backend for normalization, parsing, and remote commands
- Interactive topology canvas with inspector and diagnostics
- Sample pipeline fixtures in the repository root

What still needs work:
- Real `OE-Linux` target verification for remote flows
- Remote file browser UX
- Inspector enrichment from remote metadata
- Export, search hardening, and packaging

## Read This First

Use the documents below as the source of truth:

1. `AGENTS.md`
2. `docs/PROCESS_POLICY.md`
3. `docs/PRD.md`
4. `docs/ARCHITECTURE.md`
5. `docs/IMPLEMENTATION_PLAN.md`
6. `docs/REPOSITORY_STRUCTURE.md`
7. `docs/TECH_SPIKES.md`

## Product Summary

The app should:
- open local pipeline files and pasted text
- connect to remote `OE-Linux` devices over `SSH/SFTP`
- inspect installed GStreamer elements on the remote target
- visualize pipeline topology interactively
- show element details in an inspector panel

Locked platform direction:
- local app targets: `Windows`, `Linux`, `macOS`
- remote targets: `OE-Linux`

Recommended stack:
- desktop shell: `Tauri 2`
- core: `Rust`
- UI: `React + TypeScript`
- graph UI: `React Flow + ELK layered`
- design direction: `Miro`-inspired "Technical Canvas"

## Quick Start

Install dependencies:

```bash
npm install
```

Run the desktop app in dev mode:

```bash
npm run tauri:dev
```

Important:
- Do not treat `npm run tauri:dev` as verified just because Vite starts or the
  command keeps running.
- Dev launch is only verified when the native Tauri window appears, or the
  spawned native app process is confirmed alive without immediately crashing.

Useful supporting commands:

```bash
npm run lint
npm run build
cd src-tauri && cargo test
```

## Sample Inputs

Canonical early fixtures:
- `26_release_record_smoothing.pld.rtf`
- `27_pipmux.pld.rtf`

Reference drawings:
- `26_release_record_smoothing.png`
- `27_pipmux.png`

## Working Convention

For every substantial feature or workflow change, future agents must follow the
expert loop captured in `docs/PROCESS_POLICY.md`:

- planner
- developer
- designer
- QA

Do not call a feature complete until the user-visible flow has been verified
end to end.
