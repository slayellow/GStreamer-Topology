# Proposed Repository Structure

This structure is for the first implementation phase.

```text
/
|-- AGENTS.md
|-- README.md
|-- docs/
|   |-- PRD.md
|   |-- ARCHITECTURE.md
|   |-- IMPLEMENTATION_PLAN.md
|   |-- REPOSITORY_STRUCTURE.md
|   `-- TECH_SPIKES.md
|-- src/
|   |-- app/
|   |-- components/
|   |-- features/
|   |   |-- home/
|   |   |-- import-preview/
|   |   |-- workspace/
|   |   |-- inspector/
|   |   |-- connections/
|   |   `-- settings/
|   |-- graph/
|   |   |-- nodes/
|   |   |-- edges/
|   |   |-- layout/
|   |   `-- view-models/
|   |-- lib/
|   |-- hooks/
|   |-- styles/
|   `-- testdata/
|-- src-tauri/
|   |-- capabilities/
|   |-- src/
|   |   |-- main.rs
|   |   |-- commands/
|   |   |-- models/
|   |   |-- parser/
|   |   |-- remote/
|   |   |-- export/
|   |   |-- cache/
|   |   `-- security/
|   `-- Cargo.toml
|-- e2e/
|-- scripts/
`-- fixtures/
    |-- pipelines/
    `-- expected/
```

## Notes

- Keep sample pipelines in both the repository root and `fixtures/` only if
  there is a clear reason. Otherwise move canonical test fixtures under
  `fixtures/pipelines/` once the app scaffold exists.
- Keep Rust data models aligned with frontend TypeScript view models, but do
  not duplicate logic in both places unnecessarily.
- If the Rust side grows significantly, split reusable logic into internal
  crates later. Do not over-modularize on day one.

## Folder Purpose

### `src/features`

Feature-oriented UI modules. This keeps screens and workflows grouped by user
value instead of by low-level UI primitives.

### `src/graph`

All graph-specific logic:
- layout mapping
- custom nodes
- custom edges
- graph transformations
- selection helpers

### `src-tauri/src/parser`

Owns:
- normalization
- tokenization
- parsing
- recovery
- diagnostics

### `src-tauri/src/remote`

Owns:
- SSH connection logic
- SFTP file operations
- remote element inspection
- target environment fingerprinting

### `fixtures`

Use for:
- parser fixtures
- expected graph IR snapshots
- later remote command output samples

## Naming Guidance

- Use `workspace` for the main graph screen
- Use `inspector` for the right-side details panel
- Use `import-preview` for the pre-render validation screen
- Use `connections` for remote target management
