# Sprint 03 Week 2 Handoff

Date: 2026-04-26

## Goal

Implement the Week 2 MVP slice for linked pipeline analysis:

- Use local GStreamer metadata when available.
- Keep topology rendering working when GStreamer is unavailable.
- Show loaded pipeline text from the workspace.
- Highlight the selected element's source span.
- Allow view-only node repositioning while preserving edges.
- Surface parser diagnostics as a generation warning.
- Add a first-pass remote OE-Linux connection/load path.

## Implemented Scope

### Local GStreamer Metadata

- Added backend commands:
  - `probe_local_gstreamer`
  - `inspect_local_element`
- Local metadata uses `gst-inspect-1.0` if installed.
- Inspector now shows runtime metadata for the selected element:
  - authority
  - plugin name
  - long name
  - klass
  - description
  - pad templates
  - GStreamer properties
- If `gst-inspect-1.0` is missing or an element cannot be inspected, topology
  still renders and the inspector falls back to parsed text.

### Pipeline Source Viewer

- Added a collapsible `Pipeline 원문` panel in the workspace.
- The view model now preserves full `normalizedText`.
- Node source spans now preserve byte offsets plus line ranges.
- Selecting a node highlights the corresponding normalized pipeline text span.

### Canvas Node Movement

- React Flow nodes are now draggable through a dedicated node grip.
- Dragging only changes visual position.
- Edges remain connected to the same source/target nodes.
- Manual positions are persisted in `localStorage` by document/graph signature.
- Added layout reset control.

### Syntax Diagnostic Alert

- If parser diagnostics include warning/error severity, the workspace shows a
  non-blocking syntax alert after topology generation.
- Diagnostics panel and source panel remain available for deeper review.

### Remote OE-Linux First Pass

- Added home-screen fields for:
  - IP
  - port
  - user ID
  - password
  - remote pipeline path
- Added actions:
  - remote connection probe
  - remote topology generation through existing SFTP load
- Added backend command:
  - `inspect_remote_element`
- Remote metadata uses read-only SSH command execution with `gst-inspect-1.0`.
- Credentials are kept only in React memory for this MVP slice.

## Parser Hardening Included

Week 2 source linking depends on source spans, so the previous sample parser
gaps were fixed as part of this slice:

- `02_videotestsrc_tee_branch.pld`
  - `tee name=t t.` and later `t.` are parsed as branch references.
  - `t.` is no longer reported as a loose element token.
- `04_compositor_named_pad.pld`
  - `compositor name=mix` remains visible.
  - `mix.` and `mix.sink_1` resolve to the compositor instead of fake nodes.

## Verified

- `npm run lint`
- `npm run build`
- `cd src-tauri && cargo test`
- `command -v gst-inspect-1.0`
  - local command exists at `/Users/jshong/anaconda3/bin/gst-inspect-1.0`
- `gst-inspect-1.0 videotestsrc`
  - command succeeded and returned factory/property/pad-template output
- `npm run tauri:dev`
  - native process confirmed:
    `/Users/jshong/Downloads/Code/gstreamer-to-topology/src-tauri/target/debug/gstreamer-to-topology`

## Unverified

- Real remote `OE-Linux` target verification is still unverified.
- Specifically unverified:
  - SSH authentication against an actual target
  - SFTP remote file read against an actual target
  - remote `gst-inspect-1.0` metadata parsing against an actual target
  - failed-auth UX against an actual target
- Manual visual QA in the native window should still be performed by the user
  because this environment confirmed the process but did not record an
  interactive visual walkthrough.

## Remaining Risks

- `gst-inspect-1.0` output is human-readable and can vary by version/plugin.
  The current parser is intentionally heuristic.
- Local metadata lookup has no explicit timeout/output-size cap yet.
- Remote metadata reconnects per selected element in this first pass.
- Source highlighting targets normalized plain pipeline text positions.
- Manual layout persistence is local-only and can be reset, but is not yet part
  of an explicit saved workspace model.

## Next Smallest Step

Run manual QA in the native app:

1. Open fixtures `01` to `04`.
2. Confirm fixture `02` branch and fixture `04` compositor rendering.
3. Click nodes and open `Pipeline 원문`.
4. Drag nodes with the grip and verify edges stay connected.
5. Inspect `videotestsrc` metadata on a local GStreamer-enabled machine.
6. Test remote probe/load with a real `OE-Linux` target.
