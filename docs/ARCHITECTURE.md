# Architecture And Stack Decision

## Recommended Stack

- Desktop shell: `Tauri 2`
- Core language: `Rust`
- UI: `React + TypeScript`
- Graph interaction: `React Flow`
- Graph layout: `ELK layered`
- Remote access: `SSH + SFTP`
- Secret storage: OS secure storage through Tauri plugins or approved native
  integration

## Why This Stack

### Recommended

`Tauri 2 + Rust + React/TypeScript`

Pros:
- Smaller desktop footprint than Electron
- Strong permission and capability model
- Good fit for file I/O, parsing, SSH, caching, and export in Rust
- Access to strong node-graph UI libraries on the frontend

Tradeoffs:
- Mixed Rust and TypeScript stack
- Linux desktop testing needs extra care due to WebKitGTK variability

### Viable Alternative

`Qt 6 + QML/C++`

Use this only if the team is strongly C++/Qt-centric and willing to spend more
time on custom graph UX and packaging details.

### Not Preferred

`Electron + React/TypeScript`

Fast to start, but larger runtime and more security hardening burden for a tool
that performs remote access and local file operations.

## High-Level System

### Frontend

Responsibilities:
- Application shell
- Import screens
- Topology workspace
- Search and navigation
- Inspector
- Diagnostics display

Primary packages:
- `react`
- `typescript`
- `@xyflow/react`
- `elkjs`

### Backend

Responsibilities:
- plain-text normalization
- Tolerant parsing
- Graph IR generation
- Remote SSH/SFTP
- Metadata caching
- Export orchestration
- Secure storage integration

Primary Rust concerns:
- long-running tasks off the UI thread
- source-span preservation
- read-only remote command execution

## Data Model

### PipelineDocument

- `id`
- `source_kind` (`local_file`, `remote_file`, `pasted_text`)
- `raw_text`
- `normalized_text`
- `diagnostics[]`
- `graph`

### PipelineNode

- `id`
- `factory_name`
- `instance_name`
- `kind` (`element`, `virtual_group`, `caps`, `unknown`)
- `properties`
- `source_span`
- `metadata_ref`

### PipelinePort

- `id`
- `node_id`
- `port_kind` (`src`, `sink`, `named`, `request`)
- `port_name`

### PipelineEdge

- `id`
- `source_node_id`
- `source_port_id`
- `target_node_id`
- `target_port_id`
- `caps_label`
- `source_span`

### RemoteElementMetadata

- `factory_name`
- `long_name`
- `description`
- `plugin_name`
- `pad_templates`
- `properties`
- `target_fingerprint`

## Parsing Strategy

The parser must be tolerant, not strict.

Rules:
- Unknown elements become opaque nodes
- Unknown properties remain attached as raw key/value text
- Named links such as `eoraw.` are modeled explicitly
- Request-pad targets such as `mixer.sink_1` are modeled as ports
- Caps start as edge labels, not full nodes
- Recovery should continue after local parse failures

Avoid for MVP:
- requiring locally installed GStreamer
- requiring plugin instantiation
- hard-failing on vendor placeholders

## Rendering Strategy

Use `React Flow` for interactivity and `ELK layered` for auto-layout.

Rationale:
- React Flow supports custom nodes, handles, minimap, controls, and search
- ELK layered supports directed graphs, ports, orthogonal routing, and compound
  structures

Graphviz may be added later for:
- export
- debugging
- compatibility with GStreamer DOT workflows

It should not be the main in-app canvas renderer.

## Remote Probe Strategy

### Initial Version

- SSH for commands
- SFTP for file listing and file reads
- `gst-inspect-1.0`
- `gst-inspect-1.0 <element>`
- basic environment reads when needed

### Later Version

Add a target-side helper that emits stable JSON if CLI parsing becomes brittle.

## Security Model

- Read-only remote operations in MVP
- Explicit host-key verification
- No plaintext password storage
- Keep target metadata separate from secrets
- No remote shell string building from unsafe user text when avoidable
- Enforce output size and timeout limits on remote commands

## Packaging Plan

Initial target bundles:
- Windows: `MSI`
- macOS: `DMG`
- Linux: `AppImage` and `.deb`

Do not bundle GStreamer in v1.

Reason:
- local plugin mismatch risk
- larger package size
- remote target is already the real source of plugin truth

## Key Risks

- Vendor-specific grammar drift
- Human-readable `gst-inspect` output instability
- Large graph performance and usability
- Linux packaging and rendering consistency

## Guiding Principle

The app should be able to parse and render useful topology from text alone,
then enrich the graph with remote metadata when a target is available.
