# Product Requirements Document

## Product Name

GStreamer To Topology

## Problem Statement

Real GStreamer pipelines in embedded and OE-Linux environments become too large
to understand from plain text alone. Teams need a tool that can load pipeline
text, visualize topology, and optionally enrich it with metadata from the
actual target device where custom plugins are installed.

## Target Users

- Embedded Linux engineers
- GStreamer application developers
- Reviewers and maintainers of large multimedia pipelines
- New team members onboarding into a large pipeline codebase

## Product Goals

- Turn pipeline text into a readable, interactive topology view
- Support both local files and remote OE-Linux targets
- Help users inspect custom and vendor-specific elements
- Reduce time spent manually tracing branches, merges, and pad links

## Non-Goals

- Building a full pipeline authoring IDE in v1
- Running or controlling live pipelines in MVP
- Managing multiple targets at once in MVP
- Guaranteeing runtime correctness from static text alone

## Core User Stories

### Story A: Local Analysis

As a developer, I want to open a local pipeline file and immediately see the
topology, so that I can understand the graph faster than reading raw text.

### Story B: Remote Inspection

As a developer, I want to connect to a remote OE-Linux target, inspect the
installed elements, and open a remote pipeline file, so that I can understand
custom plugins in the environment where they actually exist.

### Story C: Debugging Branches

As a reviewer, I want to click an element and trace upstream and downstream
paths, so that I can isolate one flow inside a very large pipeline.

## MVP Scope

### Must Have

- Local file open for `.txt`, `.pld`
- Paste pipeline text
- Remote login via host, username, password
- Remote file open over SFTP
- Remote element metadata lookup via `gst-inspect-1.0`
- Tolerant parser for `gst-launch`-style syntax
- Interactive topology visualization
- Basic inspector panel
- Search and jump to node
- Parse diagnostics without blocking render

### Should Have

- Export as `PNG`, `JPG`, and later `PDF`
- Recent files and recent targets
- Branch collapse and expand
- Missing element or unresolved reference warnings

### Later

- Compare two pipeline documents
- Saved views and workspace state
- Structured remote helper for JSON metadata
- Live process attach

## Functional Requirements

### FR-1 Local Input

The app shall open local pipeline text files in supported plain-text formats.

### FR-2 Remote Connection

The app shall support connecting to one remote OE-Linux target over SSH and
browsing remote files over SFTP.

### FR-3 Remote Metadata

The app shall query installed GStreamer elements and element descriptions from
the remote target.

### FR-4 Parser

The app shall parse tolerant `gst-launch`-style pipeline text including:
- multi-line formatting
- element properties
- caps filters
- named references
- request-pad links
- branch and merge constructs
- unknown custom elements

### FR-5 Visualization

The app shall render a topology graph with:
- zoom
- pan
- minimap
- fit-to-view
- node selection
- search

### FR-6 Inspector

The app shall show node details, starting with:
- display name
- factory name
- explicit properties from source text
- incoming and outgoing links
- parse warnings for the node

### FR-7 Diagnostics

The app shall recover from partial parse failures and show diagnostics instead
of blocking the whole visualization.

## UX Principles

- Topology first, details second
- Large graphs must remain navigable
- Unknown vendor elements must still render cleanly
- Partial understanding is better than a blank error screen
- Source text and graph selection should stay linked

## Visual Direction

Use a Miro-inspired "Technical Canvas" approach:
- off-white graph canvas
- high-clarity blue for active selection
- restrained pastel grouping colors
- rounded cards and low-shadow surfaces
- compact desktop tool chrome

## Acceptance Criteria For MVP

- Opening either large sample `.pld` file renders a usable graph
- Unknown elements do not crash the parser
- Clicking a node updates the inspector
- Search can jump to a named element
- Remote connection can list and inspect at least one element on a target
- Render remains usable on a large sample graph

## Open Questions

- Whether password auth should ship first, or key auth should be first-class
- Whether caps should initially be edge labels or compact nodes
- Whether remote metadata should be pulled on demand or prefetched in batches
