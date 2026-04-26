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
- For sprint-based feature work, create or update GitHub Issues and add them to
  the GitHub Sprint Board before implementation starts. Each issue should name
  the planner/developer/designer/QA responsibilities, acceptance criteria, and
  verification checklist.
- Prefer fixture-driven development when possible.
- Prefer partial success plus diagnostics over hard failure.
- Preserve source-span mapping whenever parsing, graph IR, selection, or
  diagnostics are touched.
- Do not declare a feature complete if the user-visible flow is still
  unverified.

## Sprint Board And Branch Workflow

Use GitHub as the sprint source of truth before substantial implementation.

Sprint setup:
- When the user says to start preparing a new sprint, treat that as a board
  setup request, not only as a branch setup request.
- Keep `GStreamer Topology Sprint Board` as the parent board for the full
  backlog, history, and cross-sprint visibility.
- Create a new GitHub Project named after the active sprint, such as
  `Sprint 04`, before implementation starts.
- Link the sprint Project to the `GStreamer-Topology` repository so it appears
  from the repository's `Projects` tab.
- Add the active sprint issues, such as `#13` and `#14`, to both the parent
  board and the sprint-specific Project.
- Set the sprint-specific Project status so the user can manage that sprint
  with `Todo`, `In Progress`, and `Done`.
- Add sprint labels such as `sprint-04` so the same issues remain traceable from
  the parent board.
- If GitHub Project creation or repository linking fails, do not silently skip
  the step. Record the intended Project name, what failed, and what issue
  labels/statuses were still applied.
- Create one GitHub Issue per independently testable feature or bug.
- Write issue titles and descriptions in Korean by default.
- Use English only for code identifiers, commands, file paths, APIs, and
  GStreamer syntax.
- Each issue must include planner/developer/designer/QA responsibilities,
  acceptance criteria, and verification items.
- Each feature issue should include its own `사용자 QA 체크리스트` section.
  Avoid creating a separate checklist-only issue unless the user explicitly
  asks for an aggregate checklist.
- Expert QA results must be posted back to the corresponding feature issue as a
  Korean comment before handing the feature to the user.
- Add sprint labels such as `sprint-03` and role labels such as
  `role:developer`, `role:designer`, and `role:qa`.

Branch workflow:
- Start every sprint from a dedicated branch, preferably zero-padded for sort
  order, such as `sprint_03`.
- Do not implement sprint work directly on `main`.
- Keep the sprint branch open until implementation, automated checks,
  subagent QA, and user QA are complete.
- Open a GitHub Pull Request after user QA passes.
- Merge to `main` only after the PR reflects the completed sprint scope and
  any remaining unverified areas are explicitly documented.

Sprint closeout:
- Move completed issues to `Done` only after code is committed, pushed, and the
  user-visible QA path has passed.
- The user is expected to inspect `In Progress` issues in the active sprint
  Project, run the checklist in each issue, move passing issues to `Done`, and
  leave comments on issues that fail.
- After user QA, review the user's comments and classify each finding as either
  same-sprint rework or next-sprint backlog.
- Same-sprint rework should be fixed on the active sprint branch and returned
  to the user for another QA pass.
- Next-sprint backlog should be captured as a new issue or moved into the next
  sprint Project.
- If an issue is only partially implemented, leave it `In Progress` or move the
  remaining work into the next sprint issue.
- If a bug is discovered during user QA, create or move a follow-up issue into
  the next sprint Project, such as `Sprint 04`.

## Required Expert Subagent Loop

For every substantial feature request, create or reuse expert subagents for the
roles below.

### Team Alias Protocol

Use stable user-facing aliases for the expert loop so the user can address a
role directly during sprint work.

Default aliases:
- `Atlas`: planner
- `Forge`: developer
- `Loom`: designer
- `Beacon`: QA

Alias rules:
- Treat aliases as role contracts, not permanent process IDs. The underlying
  subagent nickname or session ID may change between runs.
- If the user says "Ask Beacon" or gives work to one alias, route that request
  to the matching role. Reuse a live matching subagent when available; otherwise
  spawn a new one with the same alias in its prompt.
- Record each alias's output in the relevant GitHub Issue when it affects
  sprint scope, implementation, design, or verification.
- The coordinating agent remains responsible for integration, final judgement,
  and explaining verified versus unverified work.
- If the user criticizes or redirects an alias, preserve the useful signal and
  feed it back into the next loop without defensiveness.

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
2. Create or update the GitHub Sprint Board items for the slice.
3. Have the planner define the smallest useful slice.
4. Have the developer implement that slice.
5. Have the designer review the interaction and visual result.
6. Have QA verify the actual user-visible behavior.
7. If QA finds issues, return the findings to the developer and repeat the loop.
8. Provide the user-facing unit test checklist before asking the user to test.
9. The coordinating agent summarizes the final state only after QA evidence is
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
