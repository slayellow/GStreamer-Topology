# Process Policy

This document defines how future agents should plan, implement, review, and
verify work in this repository.

Read this file before starting feature work.

## Core Rules

- Work in thin slices with one coherent user-visible outcome.
- Anchor every task to the current source of truth in this order:
  `docs/PRD.md`, `docs/ARCHITECTURE.md`, `docs/IMPLEMENTATION_PLAN.md`,
  `docs/REPOSITORY_STRUCTURE.md`, `docs/TECH_SPIKES.md`.
- For every substantial feature or workflow change, use the expert subagent
  loop: `planner -> developer -> designer -> QA`.
- Prefer fixture-driven development when possible.
- Prefer partial success plus diagnostics over hard failure.
- Preserve source-span mapping whenever parsing, graph IR, selection, or
  diagnostics are touched.
- Do not declare a feature complete if the user-visible flow is still
  unverified.

## Required Expert Subagent Loop

For every substantial feature request, create or reuse expert subagents for the
roles below.

### Planner

Use first when the task is cross-cutting, risky, ambiguous, or spans frontend
and backend.

Planner responsibilities:
- restate the target user-visible outcome
- identify the smallest shippable slice
- align the slice with the locked decisions and MVP scope
- define acceptance criteria
- list dependencies, risks, and stop conditions
- name the exact verification path required at the end

Planner output format:
- goal
- in-scope
- out-of-scope
- acceptance criteria
- verification steps
- stop conditions

### Developer

Developer responsibilities:
- implement only the approved slice
- keep changes minimal and directly tied to acceptance criteria
- preserve tolerant behavior and diagnostics
- avoid speculative architecture or unrelated cleanup
- keep remote behavior read-only in MVP
- leave the codebase in a runnable, reviewable state

### Designer

Designer responsibilities:
- review the workflow after it works end to end
- validate information hierarchy and graph readability
- preserve the `Miro`-inspired "Technical Canvas" direction
- improve clarity without delaying functional delivery
- avoid visual polish that undermines performance or usability

Designer review focus:
- layout clarity
- canvas readability
- inspector usefulness
- search/navigation discoverability
- state feedback for loading, warnings, and failure

### QA

QA responsibilities:
- verify the changed slice independently against the acceptance criteria
- test one happy path and one tolerant-failure path
- note regression risks and unverified areas explicitly
- block completion if verification evidence is incomplete

QA output format:
- verified
- unverified
- regressions checked
- remaining risks

## Default Feature Loop

Use this loop for feature delivery:

1. Read `AGENTS.md` and this document.
2. Have the planner define the smallest useful slice.
3. Have the developer implement that slice.
4. Have the designer review the interaction and visual result.
5. Have QA verify the actual user-visible behavior.
6. If QA finds issues, return the findings to the developer and repeat the loop.
7. The coordinating agent summarizes the final state only after QA evidence is
   present.

Do not skip the QA step for feature work.

## Slice Template

Every feature slice should be expressible in the format below.

Goal:
- one sentence describing the user-visible outcome

In scope:
- exact flows or screens being changed

Out of scope:
- adjacent improvements that must wait

Acceptance criteria:
- concrete, user-visible checks

Verification steps:
- commands to run
- UI flow to exercise
- expected visible result

Stop conditions:
- reasons to stop and hand off instead of pushing further

## Verification Standard

A task is complete only when the changed flow works from input to visible
result, not just at the function or compile level.

Minimum evidence for a normal feature slice:
- implementation compiles or builds on the changed surface
- at least one happy path is exercised end to end
- at least one tolerant-failure path is exercised when applicable
- verified and unverified areas are stated explicitly

### Tauri App Verification Rule

Do not repeat the `2026-04-22` mistake.

`npm run tauri:dev` does not count as verified just because:
- Vite started
- Rust compiled
- the command remained alive
- a "ready" or similar log appeared

`tauri:dev` counts as verified only when one of the following is true:
- the native Tauri app window actually launched
- the spawned native app process is confirmed alive without immediately
  crashing

Required evidence:
- the command that was run
- whether the native window was observed
- or the exact native process confirmation
- any limits of the environment that prevented stronger confirmation

If full launch cannot be confirmed, say `launch unverified`.

### Remote OE-Linux Verification Rule

Do not mark remote behavior as verified unless it was checked against one of
the following:
- a real `OE-Linux` target
- a controlled test target explicitly described in the handoff

If remote verification was not possible, say which part remains unverified:
- SSH auth
- SFTP listing
- remote file read
- `gst-inspect` metadata parsing

## Canonical Fixtures

Use these files first for parser and topology work:

- `26_release_record_smoothing.pld.rtf`
- `27_pipmux.pld.rtf`

Use them to validate:
- RTF normalization
- tolerant parsing
- request-pad links
- named references
- wide branch rendering
- diagnostics behavior

## Standard Verification Commands

Use the commands below when they are relevant to the slice:

```bash
npm run lint
npm run build
cd src-tauri && cargo test
npm run tauri:dev
```

Treat these as supporting evidence, not the entire verification story. The
user-visible flow still has to be exercised.

## Explicit Stop Conditions

Stop implementation and hand off when any of these are true:

- the task conflicts with the locked decisions or source-of-truth docs
- the slice has grown beyond one coherent user-visible outcome
- verification cannot be completed in the current environment
- the change would require non-read-only remote behavior in MVP
- security handling would require plaintext secrets or unsafe command
  construction
- the agent is about to start unrelated polish, refactors, or future-slice
  work

When stopping, record:
- what was completed
- what could not be verified
- the blocker or open decision
- the next smallest recommended step

## Handoff Format

End feature work with a short handoff in this format:

Completed:
- what changed

Verified:
- commands run
- user-visible flows exercised

Unverified:
- anything not proven in the current environment

Known risks:
- regressions, scale limits, or environment gaps

Next step:
- the next smallest recommended slice
