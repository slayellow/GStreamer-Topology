# Implementation Plan

## Delivery Strategy

Build in thin, verifiable slices.

The repository already has a working prototype baseline, so new work should
extend that baseline instead of restarting from scaffolding.

## Completed Baseline

These slices are already done at a prototype level:

- `Tauri 2` app scaffold
- `React + TypeScript` shell with home and workspace flows
- plain-text normalization for the sample `.pld` files
- Tolerant parser and graph IR generation
- `React Flow + ELK` topology rendering
- Basic inspector and diagnostics
- Local file and pasted-text parsing flow
- Backend SSH probe and remote pipeline load commands

These still need stronger verification or follow-up:

- live `OE-Linux` target validation
- remote file browser workflow
- remote metadata enrichment in the inspector
- search, export, and large-graph UX hardening
- packaging for `Windows`, `Linux`, and `macOS`

## Next Recommended Slice Order

### Slice 1: Real Remote Verification

Goals:
- verify SSH auth against a real `OE-Linux` target
- verify SFTP file read against a real pipeline file
- capture current gaps in remote metadata parsing

Deliverables:
- one verified remote happy path
- one written list of still-unverified remote behaviors

### Slice 2: Remote File Browser

Goals:
- add remote directory listing UI
- allow the user to browse and open a remote pipeline file
- show connection and load failures clearly

Deliverables:
- browse remote directories
- open a remote file into the workspace
- clear loading and failure states

### Slice 3: Remote Inspector Enrichment

Goals:
- parse remote `gst-inspect-1.0` output
- enrich selected elements with descriptions and properties
- cache metadata per target session

Deliverables:
- inspector shows remote metadata for at least one known element
- missing or unknown metadata degrades gracefully

### Slice 4: Search And Export

Goals:
- add search and jump
- add export to `PNG` and `JPG`
- improve graph navigation on large fixtures

Deliverables:
- search can jump to a node
- current graph can export to `PNG` and `JPG`
- large sample graphs remain usable

### Slice 5: Security And Packaging Hardening

Goals:
- integrate secure credential storage
- add host-key trust flow
- document and test packaging paths

Deliverables:
- no plaintext secret storage path
- host-key behavior is explicit
- packaging notes exist for all local app platforms

## Priority Backlog

### P0

- Real `OE-Linux` target verification
- Remote file browser
- Remote `gst-inspect` parsing and inspector enrichment
- Search and jump
- Secure secret storage

### P1

- Export `PNG` and `JPG`, then `PDF`
- Recent files and recent targets
- Branch collapse and expand
- Missing plugin and unresolved reference warnings

### P2

- Compare two pipeline documents
- Saved workspace views
- Structured remote helper for JSON metadata
- Live process attach

## Immediate Next Tasks

Start here in the next implementation turn unless the user redirects:

1. Run the planner/developer/designer/QA loop for the remote verification slice
2. Verify SSH and SFTP against a real `OE-Linux` target if one is available
3. Document exactly what part of the remote flow is still unverified
4. Build the remote file browser only after the backend behavior is confirmed

## Exit Criteria By Milestone

### Milestone A: Local MVP Core

Done when:
- local sample files open
- parser returns graph IR
- graph renders
- inspector works
- diagnostics show

Status:
- prototype complete
- keep hardening, but do not restart this milestone

### Milestone B: Remote Read-Only

Done when:
- target can connect
- remote file can open
- remote `gst-inspect` data can enrich the inspector

Status:
- partially implemented
- still needs live target verification and UI completion

### Milestone C: Stabilized MVP

Done when:
- packaging path exists for all target platforms
- credentials are stored securely
- graph remains usable on large sample pipelines

Status:
- not complete

## Testing Plan

- Unit tests for normalization
- Fixture tests for parser
- UI verification for local import and selection flows
- Remote verification against a real `OE-Linux` target
- Regression checks on tolerant-failure paths

Use `docs/PROCESS_POLICY.md` for the mandatory handoff and verification format.

## Suggested Next Demo

The next convincing demo should show:

- connect to one real `OE-Linux` target
- browse to a remote pipeline file
- open it in the workspace
- render topology
- click a node
- show remote element details in the inspector
