# Technical Spike List

This list exists to reduce implementation risk before building too much
product surface area.

## Spike 01: Plain Pipeline Text Normalization

Question:
- Can the sample `.pld` files be normalized into clean pipeline text without
  losing meaningful symbols?

Why:
- The current real samples are large plain pipeline files with custom elements,
  named references, caps strings, and placeholders.

Success criteria:
- Both sample files normalize into readable pipeline text
- No branch syntax or caps text is accidentally removed
- Plain text whitespace cleanup does not create parser warnings

Output:
- plain-text normalization utility
- fixture tests

## Spike 02: Tolerant Pipeline Parser

Question:
- Can one parser handle the current sample files, including custom elements and
  non-trivial links, without depending on local GStreamer?

Why:
- This is the core product risk.

Success criteria:
- Supports unknown vendor elements
- Supports named references such as `eoraw.`
- Supports request-pad links such as `mixer.sink_1`
- Emits diagnostics for unresolved pieces instead of crashing

Output:
- parser module
- graph IR
- parser fixtures and expected snapshots

## Spike 03: Graph Layout On Real Samples

Question:
- Can `React Flow + ELK layered` render the real sample graphs in a readable
  way without heavy manual layout logic?

Why:
- The sample topology is already very wide and branch-heavy.

Success criteria:
- Graph is readable at default zoom
- Branches do not overlap excessively
- Selection and minimap stay responsive

Output:
- layout adapter
- first workspace prototype

## Spike 04: Remote SSH/SFTP Probe

Question:
- Can the app connect to a representative OE-Linux target and read files plus
  `gst-inspect` output reliably?

Why:
- Remote metadata is a key differentiator.

Success criteria:
- Connect to one target
- List files in one configured directory
- Read one pipeline file
- Inspect one known element

Output:
- target profile model
- connection test
- remote open proof

## Spike 05: `gst-inspect` Parsing Strategy

Question:
- Is parsing human-readable `gst-inspect-1.0` output stable enough for MVP, or
  do we need a structured helper earlier?

Why:
- CLI output can vary by version or environment.

Success criteria:
- Extract long name, description, pad templates, and properties for at least a
  small test set of elements
- Record known parsing failure modes

Output:
- metadata parser
- fallback and error strategy

## Spike 06: Secure Secret Storage

Question:
- What is the cleanest way to store target credentials while keeping password
  storage out of plaintext config?

Why:
- Remote login is part of MVP.

Success criteria:
- Credentials stored through OS secure storage or an approved secure layer
- Connection metadata separated from secrets

Output:
- secure storage integration note
- implementation recommendation

## Spike 07: Export Pipeline Views

Question:
- What is the best path for exporting a current graph view to `PNG` and `JPG`,
  and later `PDF`?

Why:
- Export is a strong review and documentation feature.

Success criteria:
- Raster export preserves node labels and edge readability
- Export works from the rendered graph, not just from raw text

Output:
- export path decision
- first PNG/JPG export

## Spike 08: Large Graph Performance

Question:
- At what graph size does the workspace become too slow, and what simplification
  strategies should be added first?

Why:
- UX will likely become the bottleneck before parsing does.

Success criteria:
- Measure render and interaction performance on the sample graphs
- Identify first optimizations:
  - semantic zoom
  - branch collapse
  - viewport culling

Output:
- performance notes
- threshold estimates

## Recommended Spike Order

1. Plain pipeline text normalization
2. Tolerant parser
3. Graph layout on real samples
4. Remote SSH/SFTP probe
5. `gst-inspect` parsing
6. Secure secret storage
7. Export
8. Performance
