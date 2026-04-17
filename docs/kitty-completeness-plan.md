# Kitty graphics completeness plan

Status: next-phase planning note for the current proof branch.

This document covers the work after the current stabilized proof milestone.
It is intentionally about **capability completeness**, not upstream slicing.

Related:
- `docs/kitty-graphics-checklist.md`
- `docs/kitty-image-rearchitecture-sketch.md`
- `docs/kitty-test-strategy.md`

## Why this phase exists

The branch is now past basic feasibility:
- in-core kitty support works
- real apps work (`chafa`, `ratatui-image` / `flotilla`, `yazi` with the current workaround)
- placeholder flow and one-dimensional explicit sizing have been stabilized materially
- targeted regression tests now protect the most recent high-value fixes

That means the next question is no longer:
- "can Zellij support kitty graphics at all?"

It is:
- "what capability level should Zellij aim to support so it does not become the blocker?"

## Guiding principle

We should prioritize completeness by two lenses at the same time:

1. **What real apps use today**
   - practical compatibility
   - what modern TUI/image apps are likely to depend on

2. **What capabilities would be costly if Zellij became the limiting factor**
   - even if not every current app uses them yet
   - preserve a path for a meaningful kitty capability level

The goal is to avoid landing in a "toy subset" of kitty graphics support.

## Current capability floor already proven

The current branch already demonstrates a meaningful base capability level:
- inline PNG and RGBA transmit
- chunking
- explicit placements
- placeholder / virtual placements
- one-dimensional sizing (`c+r`, `c only`, `r only`)
- basic query support sufficient for real autodetection
- delete-all-visible
- clear / reset / alt-screen lifecycle
- pane composition and clipping
- multi-app proof in Kitty/Ghostty-class environments

This is a strong proof baseline.

## Completeness priorities

## Tier 1: highest-value next proof surface

These are the first things to expand in the smoke harness and likely the first semantics to tighten in code.

### 1. Targeted delete semantics

Current status:
- `a=d,d=A` works
- delete by protocol image id / placement id works for the current proof branch
- smoke coverage now exercises targeted deletion with multiple visible placements

Remaining follow-up:
- decide how far to go on additional delete selectors beyond current real-app needs
- keep replacement and deletion interactions covered as the architecture settles

Why this matters:
- natural next lifecycle step after delete-all-visible
- validates logical placement ownership and identity handling
- avoids Zellij being limited to brute-force delete semantics only

### 2. Multi-placement and replacement semantics

Current status:
- same asset can be transmitted once and displayed in multiple placements
- same protocol image id can be reused intentionally within pane-local identity rules
- targeted deletion of one placement without disturbing another now works in the smoke harness
- pane-local protocol identity vs Zellij-owned internal identity is now part of the working proof surface

Why this matters:
- already a known source of real bugs on this branch
- central to multiplexer correctness
- easy to regress during cleanup if not explicitly exercised

### 3. Erase interactions

Current status:
- overwrite cleanup exists
- `ESC[2J]`, reset, and alt-screen lifecycle are handled
- erase-in-line interactions (`EL 0`, `EL 1`, `EL 2`) now work for placeholder-backed content in the proof branch
- smoke coverage now exercises overwrite and line-erase cases directly

Remaining follow-up:
- region-clear style interactions as feasible
- broader lifecycle audit if a real app exposes another gap

Why this matters:
- placeholder mode is text-flow integrated
- erase behavior is common and easy to regress
- this is an important completeness step before claiming broad lifecycle coherence

### 4. Resize and reflow behavior

Current status:
- improved materially
- resize/reflow smoke coverage exists
- damage-aware kitty redraw now participates in changed-rect-driven frames
- `Output` now has an explicit kitty damage-redraw path separate from the persistent visible-scene path
- live behavior is now much closer to frame-perfect in current testing

Remaining follow-up:
- decide whether the current coarse row-intersection policy is the right long-term damage-redraw boundary
- keep refining resize/reflow behavior near viewport boundaries and under more unusual terminal interactions
- treat this as architectural polish now, not basic feasibility

Why this matters:
- this was the biggest remaining correctness hole
- it is still the most architecturally informative open area
- worth understanding even where exact host-terminal behavior differs

## Tier 2: capability-level support to assess soon

These may or may not all be implemented immediately, but we should know clearly where the branch stands.

### 5. `f=24` RGB payloads

Current status:
- still open

Why this matters:
- low conceptual complexity
- removes an avoidable format gap
- helps avoid an unnecessarily narrow capability claim

### 6. More complete query behavior

Current status:
- enough `a=q` support exists for current real-world autodetection to work
- not yet a full spec-audit of query / response behavior

Remaining targets:
- `a=q` response shape audit
- echoed ids where expected
- basic OK / error response behavior
- quiet-mode semantics audit as needed

Why this matters:
- autodetection and backend selection depend on this class of feature
- partial query behavior can leave apps stuck on fallback paths forever

### 7. Scroll-region / page-margin interactions

Current status:
- still open
- smoke probing now suggests region-local scrolling is a real semantic gap for current Zellij kitty image behavior
- direct-terminal behavior in this area also appears murky enough that it is not currently a strong practical reference point

Current conclusion:
- worth documenting as incomplete
- not currently a high-priority completeness blocker absent a real app or stronger practical demand signal

Why this matters:
- not every app uses them directly for images, but terminals and TUIs can interact with them
- useful as a capability-level check that image flow is not overfit to whole-viewport assumptions

## Tier 3: consciously deferred for now

These are real protocol features, but they should not dominate the next completeness pass unless a real app forces the issue.

- file / tempfile / shared-memory transports (`t=f`, `t=t`, `t=s`)
- compression (`o=z`)
- relative placements (`P=`, `Q=`, `H=`, `V=`)
- quota / eviction semantics
- animation

## Smoke harness expansion plan

The smoke harness should continue to be the main scout before broader implementation work.

### Current smoke coverage

The smoke harness now includes the originally planned high-value completeness stages:

#### A. Targeted delete stage
- creates multiple visible placements
- deletes one by image id / placement id
- verifies the intended placement is the one that disappears

#### B. Multi-placement / replacement stage
- transmits once, displays multiple times
- exercises the asset/placement split and placement identity handling

#### C. Erase interaction stage
- covers placeholder-backed content followed by overwrite
- covers line clear / erase-to-end / erase-to-beginning cases
- verifies placeholder cleanup matches text-flow expectations

#### D. Resize / reflow stage
- renders explicit and placeholder examples
- shrinks and expands width
- acts as a comparative coherence probe for clipping, anchoring, and redraw behavior

## Implementation order recommendation

Recommended next sequence:

1. sync docs/checklists with the now-completed smoke and regression work
2. decide whether kitty damage-redraw selection should stay as coarse row intersection or evolve into a more explicit redraw-policy layer
3. use current stability to choose the next capability surface intentionally:
   - scroll-region / page-margin behavior if a real app or stronger demand signal appears
   - otherwise other protocol-surface work with clearer practical value
4. treat scroll-region / page-margin behavior as documented-but-deferred unless a stronger use case appears

This keeps the harness acting as a behavioral checkpoint, rather than guessing completeness priorities from the spec alone.

## Success criterion for the completeness phase

A reasonable target is that Zellij can credibly claim a modern, useful kitty graphics capability tier:
- inline image transmit works for common formats in practice
- explicit and placeholder placement modes work
- one-dimensional sizing works
- multi-placement and targeted lifecycle operations work
- clear / reset / alt-screen / erase interactions are coherent
- query behavior is sufficient for backend selection
- resize / reflow behavior is understood and not obviously Zellij-limiting

This branch is now materially closer to that target than when this document was first drafted; the remaining work is no longer basic completeness scaffolding so much as selecting the next protocol surface and polishing the redraw policy boundary.

One example of a now-better-understood deferred area is scroll-region/page-margin behavior: the exploratory smoke probe found a real current limitation, but also suggested weak practical demand and weak direct-terminal reference behavior, so it should not be treated as a current blocker by default.

That does **not** require complete protocol parity before moving on.
But it does require enough completeness that Zellij is not the reason an app must avoid kitty graphics support.

## Relationship to upstreaming

This phase is intentionally separate from upstream slicing.

The current priority is:
- deepen confidence in the meaningful capability surface
- avoid defining too-low a support ceiling
- keep learning what the architecture must preserve

Only after that should the work be aggressively chopped for upstream review.
